use std::collections::{HashMap, HashSet};
use godot::prelude::*;
use std::fs::File;
use std::io::{BufReader, BufRead};
use rand::seq::SliceRandom;
use rand::rng;

#[derive(GodotClass)]
#[class(base=Node)]
struct Bayes{
    base: Base<Node>,
}

#[godot_api]
impl INode for Bayes {
    fn init(base: Base<Node>) -> Self{
        Self { base }
    }
}

#[derive(Debug, Clone)]
struct TweetEtiquete {
    contenu: String,
    etiquette: i32, //4 = positif, 2 = neutre, 0 = négatif
}

#[derive(Debug)]
struct BayesModel {
    log_prior: HashMap<i32, f64>,
    log_prob: HashMap<i32, HashMap<String, f64>>,
    vocab_taille: usize,
    total_mots_par_classe: HashMap<i32, usize>
}

#[derive(Debug, Clone, Copy)]
enum VoteType {
    Laplace,
    AddAlpha,
}

#[godot_api]
impl Bayes{
    #[func]
    fn bayes_execute(&mut self, path: GString, tweet: GString, type_vote: i64) -> GString {
        let path_str = path.to_string();
        let tweet_str = tweet.to_string();
        let vote_type = match type_vote as usize {
            1 => VoteType::AddAlpha,
            _ => VoteType::Laplace,
        };
        let data = match charger_donnees(&path_str){
            Ok(v) => v,
            Err(e) => {
                self.signals().log_sent().emit(&format!("Erreur chargement données : {}", e));
                return GString::from("ERREUR");
            }
        };
        let model = BayesModel::new(&data, vote_type);
        match model.classifier(&tweet_str) {
            Some(classe) => {
                let res = match classe {
                    4 => "POSITIF",
                    2 => "NEUTRE",
                    0 => "NÉGATIF",
                    _ => "INCONNU",
                };
                GString::from(res)
            }
            None => {
                self.signals().log_sent().emit(&GString::from("Impossible de classifier le tweet"));
                GString::from("ERREUR")
            }
        }
    }

    #[func]
    fn bayes_evaluate(&mut self, path: GString, type_vote: i64) -> GString {
        let path_str = path.to_string();
        let vote_type = match type_vote as usize {
            1 => VoteType::AddAlpha,
            _ => VoteType::Laplace,
        };
        let all = match charger_donnees(&path_str) {
            Ok(v) => v,
            Err(e) => {
                self.signals().log_sent().emit(&format!("Erreur chargement données : {}", e));
                return GString::from("ERREUR");
            }
        };
        let (train, test) = diviser_donnees_stratifiee(&all, 2.0 / 3.0);
        if train.is_empty() || test.is_empty() {
            self.signals().log_sent().emit(&GString::from("Base d'entraînement ou test vide"));
            return GString::from("ERREUR");
        }
        let model = BayesModel::new(&train, vote_type);
        let mut matrice_confusion = [[0usize; 3]; 3];
        let index_map : HashMap<i32, usize> = [(0,0), (2,1), (4,2)].iter().cloned().collect();
        for tweet in &test {
            if let Some(pred) = model.classifier(&tweet.contenu) {
                if let (Some(&idx_reel), Some(&idx_est)) = (index_map.get(&tweet.etiquette), index_map.get(&pred)){
                    matrice_confusion[idx_reel][idx_est] += 1;
                }
            }
        }
        GString::from(format_matrice_confusion(&matrice_confusion))

    }

    #[signal]
    fn log_sent(message: GString);
}

impl BayesModel {
    fn new(data: &[TweetEtiquete], vote: VoteType) -> Self {
        let mut class_counts: HashMap<i32, usize> = HashMap::new();
        for t in data {
            *class_counts.entry(t.etiquette).or_insert(0) += 1;
        }

        let mut word_counts: HashMap<i32, HashMap<String, usize>> = HashMap::new();
        let mut vocab: HashSet<String> = HashSet::new();

        for t in data {
            let tokens = tokeniser_tweet(&t.contenu);
            let map = word_counts.entry(t.etiquette).or_insert_with(HashMap::new);
            for w in &tokens {
                *map.entry(w.clone()).or_insert(0) += 1;
                vocab.insert(w.clone());
            }
        }

        let alpha = match vote {
            VoteType::Laplace => 1.0,
            VoteType::AddAlpha => 0.5,
        };

        let total_documents = class_counts.values().sum::<usize>() as f64;
        let mut log_prior = HashMap::new();
        for (&cls, &cnt) in &class_counts {
            log_prior.insert(cls, (cnt as f64 / total_documents).ln());
        }

        let mut log_prob: HashMap<i32, HashMap<String, f64>> = HashMap::new();
        let vocab_taille = vocab.len() as f64;
        for (&cls, map) in &word_counts {
            let class_total: usize = map.values().sum();
            let denom = class_total as f64 + alpha * vocab_taille;
            let mut ll_map: HashMap<String, f64> = HashMap::new();
            for (w, &cnt) in map {
                let p = (cnt as f64 + alpha) / denom;
                ll_map.insert(w.clone(), p.ln());
            }
            log_prob.insert(cls,ll_map);
        }

        let mut total_mots_par_classe = HashMap::new();
        for (&cls, map) in &word_counts {
            let total: usize = map.values().sum();
            total_mots_par_classe.insert(cls,total);
        }
        Self {
            log_prior,
            log_prob,
            vocab_taille: vocab.len(),
            total_mots_par_classe
        }
    }

    fn classifier(&self, tweet: &str) -> Option<i32> {
        let tokens = tokeniser_tweet(tweet);
        let mut scores: HashMap<i32, f64> = HashMap::new();
        for (&cls, &prior) in &self.log_prior {
            let mut score = prior;
            if let Some(ll_map) = self.log_prob.get(&cls) {
                for w in &tokens {
                    let log_lk = ll_map.get(w).cloned().unwrap_or_else(|| {
                        let class_total = *self.total_mots_par_classe.get(&cls).unwrap_or(&0);
                        let denom = class_total as f64 + 1.0 * self.vocab_taille as f64;
                        (1.0 / denom).ln()
                    });
                    score += log_lk;
                }
            }
            scores.insert(cls, score);
        }
        scores.into_iter().max_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).map(|(c,_)| c)
    }
}

fn charger_donnees(chemin: &str) -> Result<Vec<TweetEtiquete>, Box<dyn std::error::Error>> {
    let f = File::open(chemin)?;
    let reader = BufReader::new(f);
    let mut donnees = Vec::new();
    for (i,l) in reader.lines().enumerate() {
        let l = l?;
        if i == 0 {
            continue;
        }
        let cols: Vec<&str> = l.split(',').collect();
        if cols.len() >= 2 {
            if let Ok(e) = cols[0].parse::<i32>() {
                let contenu = cols[1..].join(",");
                donnees.push(TweetEtiquete {
                    contenu,
                    etiquette: e,
                });
            }
        }
    }
    Ok(donnees)
}

fn tokeniser_tweet(tweet: &str) -> HashSet<String> {
    tweet.split_whitespace().map(|w| w.to_lowercase()).collect()
}

fn diviser_donnees_stratifiee(donnees: &[TweetEtiquete], ratio_train: f64) -> (Vec<TweetEtiquete>, Vec<TweetEtiquete>){
    let mut rng = rng();
    let mut par_classe: HashMap<i32, Vec<TweetEtiquete>> = HashMap::new();
    for t in donnees {
        par_classe.entry(t.etiquette).or_insert_with(Vec::new).push(t.clone());
    }
    let mut entrainement = Vec::new();
    let mut test = Vec::new();
    for (_, mut vec) in par_classe {
        vec.shuffle(&mut rng);
        let split = (vec.len() as f64 * ratio_train) as usize;
        entrainement.extend_from_slice(&vec[..split]);
        test.extend_from_slice(&vec[split..]);
    }
    entrainement.shuffle(&mut rng);
    test.shuffle(&mut rng);
    (entrainement, test)
}

fn format_matrice_confusion(matrice: &[[usize; 3]]) -> String {
    // Indices : 0 = négatif, 1 = neutre, 2 = positif
    let n_pos_reel = matrice[2][0] + matrice[2][1] + matrice[2][2];
    let n_neg_reel = matrice[0][0] + matrice[0][1] + matrice[0][2];
    let n_neu_reel = matrice[1][0] + matrice[1][1] + matrice[1][2];
    let n_pos_estime = matrice[0][2] + matrice[1][2] + matrice[2][2];
    let n_neg_estime = matrice[0][0] + matrice[1][0] + matrice[2][0];
    let n_neu_estime = matrice[0][1] + matrice[1][1] + matrice[2][1];
    let total = n_pos_reel + n_neg_reel + n_neu_reel;
    format!(
        "[table=5]\n\
        [cell]Réel/Estimé[/cell][cell]Positive[/cell][cell]Négatif[/cell][cell]Neutre[/cell][cell]Total réel[/cell]\n\
        [cell]Positive[/cell][cell]{}[/cell][cell]{}[/cell][cell]{}[/cell][cell]{}[/cell]\n\
        [cell]Négatif[/cell][cell]{}[/cell][cell]{}[/cell][cell]{}[/cell][cell]{}[/cell]\n\
        [cell]Neutre[/cell][cell]{}[/cell][cell]{}[/cell][cell]{}[/cell][cell]{}[/cell]\n\
        [cell]Total estimé[/cell][cell]{}[/cell][cell]{}[/cell][cell]{}[/cell][cell]{}[/cell]\n\
        [/table]",
        // Ligne Positive réelle
        matrice[2][2], // TP_pos
        matrice[2][0], // FN_pos → négatif
        matrice[2][1], // FN_pos → neutre
        n_pos_reel,
        // Ligne Négative réelle
        matrice[0][2], // FP_neg → positif
        matrice[0][0], // TP_neg
        matrice[0][1], // FN_neg → neutre
        n_neg_reel,
        // Ligne Neutre réelle
        matrice[1][2], // FP_neu → positif
        matrice[1][0], // FP_neu → négatif
        matrice[1][1], // TP_neu
        n_neu_reel,
        // Totaux estimés
        n_pos_estime,
        n_neg_estime,
        n_neu_estime,
        total
    )
}