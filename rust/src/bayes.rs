use std::collections::{HashMap, HashSet};
use godot::prelude::*;
use std::fs::File;
use std::io::{BufReader, BufRead};
use rand::seq::SliceRandom;
use rand::rng;

use crate::bayes::ngram::NgramMode;
use crate::bayes::representation::Representation;
use crate::bayes::smoothing::VoteType;

mod ngram;
mod smoothing;
mod representation;

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
    etiquette: i32,
}


#[derive(Debug)]
struct BayesModel {
    log_prior: HashMap<i32, f64>,
    log_prob: HashMap<i32, HashMap<String, f64>>,
    vocab: HashSet<String>,
    vocab_taille: usize,
    total_mots_par_classe: HashMap<i32, usize>,
    representation: Representation,
    ngram_mode: NgramMode,
    alpha: f64,
}

#[godot_api]
impl Bayes{
    #[func]
    fn bayes_execute(&mut self, path: GString, tweet: GString, type_vote: i64, type_representation: i64, ngram_type: i64) -> GString {
        let path_str = path.to_string();
        let tweet_str = tweet.to_string();
        
        let vote_type = VoteType::from(type_vote);

        let representation = Representation::from(type_representation);

        let ngram_mode = NgramMode::from(ngram_type);

        let data = match charger_donnees(&path_str){
            Ok(v) => v,
            Err(e) => {
                self.signals().log_sent().emit(&format!("Erreur chargement données : {}", e));
                return GString::from("ERREUR");
            }
        };

        let model = BayesModel::new(&data, vote_type, representation, ngram_mode);
        
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
    fn bayes_evaluate(&mut self, path: GString, type_vote: i64, type_representation: i64, ngram_type: i64) -> GString {
        let path_str = path.to_string();
        
        let vote_type = VoteType::from(type_vote);

        let representation = Representation::from(type_representation);

        let ngram_mode = NgramMode::from(ngram_type);

        let all = match charger_donnees(&path_str) {
            Ok(v) => v,
            Err(e) => {
                self.signals().log_sent().emit(&format!("Erreur chargement données : {}", e));
                return GString::from("ERREUR");
            }
        };

        // Division du dataset : 2/3 entraînement, 1/3 test
        let (train, test) = diviser_donnees_stratifiee(&all, 2.0 / 3.0);
        if train.is_empty() || test.is_empty() {
            self.signals().log_sent().emit(&GString::from("Base d'entraînement ou test vide"));
            return GString::from("ERREUR");
        }

        let model = BayesModel::new(&train, vote_type, representation, ngram_mode);
        
        let mut matrice_confusion = [[0; 3]; 3];
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
    fn new(data: &[TweetEtiquete], vote: VoteType, representation: Representation, ngram_mode: NgramMode) -> Self {

        // Comptage
        let mut class_counts: HashMap<i32, usize> = HashMap::new();
        for t in data {
            *class_counts.entry(t.etiquette).or_insert(0) += 1;
        }

        let mut word_counts: HashMap<i32, HashMap<String, usize>> = HashMap::new();
        let mut vocab: HashSet<String> = HashSet::new();

        for t in data {
            let tokens = ngram_mode.tokeniser_tweet(&t.contenu);
            
            // Selon le mode, on garde soit tous les mots (Fréquence), soit les uniques (Présence)
            let tokens_to_count: Vec<String> = representation.tokens_to_count(tokens);
            
            // Remplissage de la matrice de comptage
            let map = word_counts.entry(t.etiquette).or_insert_with(HashMap::new);
            for w in tokens_to_count {
                *map.entry(w.clone()).or_insert(0) += 1;
                vocab.insert(w);
            }
        }

        // Définition du paramètre de lissage
        let alpha = f64::from(vote);
        
        // Calcul des Priors
        let total_documents = class_counts.values().sum::<usize>() as f64;
        let mut log_prior = HashMap::new();
        for (&cls, &cnt) in &class_counts {
            log_prior.insert(cls, (cnt as f64 / total_documents).ln());
        }
        
        // Calcul des Likelihoods
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
            log_prob.insert(cls, ll_map);
        }

        let mut total_mots_par_classe = HashMap::new();
        for (&cls, map) in &word_counts {
            let total: usize = map.values().sum();
            total_mots_par_classe.insert(cls, total);
        }

        Self {
            log_prior,
            log_prob,
            vocab : vocab.clone(),
            vocab_taille: vocab.len(),
            total_mots_par_classe,
            representation,
            ngram_mode,
            alpha,
        }
    }

    // Retourne la classe ayant le score (log-probabilité) le plus élevé
    fn classifier(&self, tweet: &str) -> Option<i32> {
        let tokens = self.ngram_mode.tokeniser_tweet(tweet);
        
        let tokens_to_score: Vec<String> = self.representation.tokens_to_count(tokens);

        let mut scores: HashMap<i32, f64> = HashMap::new();

        for (&cls, &prior) in &self.log_prior {
            let mut score = prior;

            // On récupère les infos de la classe
            let ll_map = self.log_prob.get(&cls).unwrap();
            let class_total = *self.total_mots_par_classe.get(&cls).unwrap_or(&0);
            
            // Calcul de la probabilité par défaut pour un mot inconnu dans cette classe (mais connu du vocabulaire global)
            let denom = class_total as f64 + self.alpha * self.vocab_taille as f64;
            let log_defaut = (self.alpha / denom).ln();

            for w in &tokens_to_score {
                // 1. Si le mot n'est pas dans le vocabulaire global d'entrainement, on l'ignore totalement
                if !self.vocab.contains(w) {
                    continue; 
                }

                // 2. On cherche la proba du mot dans la classe, sinon on utilise la proba lissée
                let log_lk = ll_map.get(w).cloned().unwrap_or(log_defaut);
                
                score += log_lk;
            }
            scores.insert(cls, score);
        }

            // on renvoie une classe par défaut ou aléatoire si égalité parfaite
            scores.into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(c,_)| c)
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
        let colonnes: Vec<&str> = l.split(',').collect();
        if colonnes.len() >= 2 {
            if let Ok(e) = colonnes[0].parse::<i32>() {
                let contenu_raw = colonnes[1..].join(",");
                let contenu = contenu_raw.trim_matches('"').to_string();
                donnees.push(TweetEtiquete {
                    contenu,
                    etiquette: e,
                });
            }
        }
    }
    Ok(donnees)
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

// Formate la matrice de confusion selon le format demandé
fn format_matrice_confusion(matrice: &[[i32; 3]]) -> String {
    let n_pos_reel = matrice[2][0] + matrice[2][1] + matrice[2][2]; // Réel: Positif (index 2)
    let n_neg_reel = matrice[0][0] + matrice[0][1] + matrice[0][2]; // Réel: Négatif (index 0)
    let n_neu_reel = matrice[1][0] + matrice[1][1] + matrice[1][2]; // Réel: Neutre (index 1)
    
    let n_pos_estime = matrice[0][2] + matrice[1][2] + matrice[2][2]; // Estimé: Positif
    let n_neg_estime = matrice[0][0] + matrice[1][0] + matrice[2][0]; // Estimé: Négatif
    let n_neu_estime = matrice[0][1] + matrice[1][1] + matrice[2][1]; // Estimé: Neutre
    
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
        matrice[2][0], // FN_pos→neg
        matrice[2][1], // FN_pos→neu
        n_pos_reel,
        
        // Ligne Négative réelle
        matrice[0][2], // FP_neg→pos
        matrice[0][0], // TP_neg
        matrice[0][1], // FN_neg→neu
        n_neg_reel,
        
        // Ligne Neutre réelle
        matrice[1][2], // FP_neu→pos
        matrice[1][0], // FP_neu→neg
        matrice[1][1], // TP_neu
        n_neu_reel,
        
        // Totaux estimés
        n_pos_estime,
        n_neg_estime,
        n_neu_estime,
        total
    )
}