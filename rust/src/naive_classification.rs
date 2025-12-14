use std::collections::{HashSet, HashMap};
use godot::prelude::*;
use std::fs::File;
use std::io::{BufReader, BufRead};
use rand::seq::SliceRandom;
use rand::rng;

#[derive(GodotClass)]
#[class(base=Node)]
struct Naive {
    base: Base<Node>,
}

#[godot_api]
impl INode for Naive {
    fn init(base: Base<Node>) -> Self {
        Self { base }
    }
}

#[derive(Debug, Clone)]
struct TweetEtiquete {
    contenu: String,
    etiquette: i32, // 4=positif, 2=neutre, 0=négatif
}

#[godot_api]
impl Naive {

    #[func]
    fn naive_execute(
        &mut self, 
        path_pos: GString, 
        path_neg: GString, 
        tweet_a_classifier: GString, 
        weight: f64
    ) -> GString {
        let tweet_str = tweet_a_classifier.to_string();
        
        let pos_words = match charger_dictionnaire(&path_pos.to_string()) {
            Ok(w) => w,
            Err(_) => return GString::from("ERREUR: Chargement dico positif"),
        };
        
        let neg_words = match charger_dictionnaire(&path_neg.to_string()) {
            Ok(w) => w,
            Err(_) => return GString::from("ERREUR: Chargement dico négatif"),
        };

        let result = analyser_tweet(&tweet_str, &pos_words, &neg_words, weight as f32);

        let resultat_str = match result {
            4 => "POSITIF",
            2 => "NEUTRE",
            0 => "NÉGATIF",
            _ => "INCONNU"
        };
        
        GString::from(resultat_str)
    }

    #[func]
    fn naive_evaluate(
        &mut self, 
        path_data: GString, 
        path_pos: GString, 
        path_neg: GString, 
        weight: f64
    ) -> GString {
        // 1. Chargement des dictionnaires
        let pos_words = match charger_dictionnaire(&path_pos.to_string()) {
            Ok(w) => w,
            Err(e) => {
                self.signals().log_sent().emit(&GString::from(format!("Erreur dico pos: {}", e)));
                return GString::from("ERREUR");
            }
        };
        
        let neg_words = match charger_dictionnaire(&path_neg.to_string()) {
            Ok(w) => w,
            Err(e) => {
                self.signals().log_sent().emit(&GString::from(format!("Erreur dico neg: {}", e)));
                return GString::from("ERREUR");
            }
        };

        // 2. Chargement des données complètes
        let all_data = match charger_donnees(&path_data.to_string()) {
            Ok(d) => d,
            Err(e) => {
                self.signals().log_sent().emit(&GString::from(format!("Erreur data: {}", e)));
                return GString::from("ERREUR");
            }
        };

        if all_data.is_empty() {
             return GString::from("Dataset vide");
        }

        // 3. Division Stratifiée (2/3 Train, 1/3 Test) (même si on utilise pas le 2/3 train)
        let (_train, test) = diviser_donnees_stratifiee(&all_data, 2.0 / 3.0);

        if test.is_empty() {
            return GString::from("Erreur: Jeu de test vide après division");
        }

        // 4. Évaluation sur le set de test
        let mut matrice_confusion = [[0; 3]; 3];
        let index_map: HashMap<i32, usize> = [(0, 0), (2, 1), (4, 2)].iter().cloned().collect();

        for tweet in &test {
            let classe_estimee = analyser_tweet(&tweet.contenu, &pos_words, &neg_words, weight as f32);
            
            if let (Some(&idx_reel), Some(&idx_estime)) = (
                index_map.get(&tweet.etiquette),
                index_map.get(&classe_estimee)
            ) {
                matrice_confusion[idx_reel][idx_estime] += 1;
            }
        }

        let resultat = format_matrice_confusion(&matrice_confusion);
        GString::from(resultat)
    }

    #[signal]
    fn log_sent(message: GString);
}

fn analyser_tweet(tweet: &str, pos_set: &HashSet<String>, neg_set: &HashSet<String>, weight: f32) -> i32 {
    let mut positives: u32 = 0;
    let mut negatives: u32 = 0;

    let words = tweet.split(|c: char| !c.is_alphanumeric())
                     .filter(|s| !s.is_empty());

    for word in words {
        let clean_word = word.to_lowercase();
        
        if pos_set.contains(&clean_word) {
            positives += 1;
        } else if neg_set.contains(&clean_word) {
            negatives += 1;
        }
    }

    compute_polarity_with_weight(negatives, positives, weight)
}

fn compute_polarity_with_weight(negatives: u32, positives: u32, weight: f32) -> i32 {
    let f_negatives = negatives as f32;
    let f_positives = positives as f32;
    let f_total = f_negatives + f_positives;

    if f_total == 0.0 {
        return 2;
    }
    
    let pos_ratio = f_positives / f_total;

    // Logique heuristique : Si ratio > poids, c'est positif.
    // Note : si weight est 1.0, il faut un ratio STICTEMENT supérieur, donc impossible (max 1.0).
    // J'ai gardé ta logique "|| pos_ratio == 1.0" pour gérer le cas où weight = 1.0
    if pos_ratio > weight || pos_ratio == 1.0 { 
        return 4; 
    }
    
    let neg_ratio = f_negatives / f_total;
    if neg_ratio > weight || neg_ratio == 1.0 { 
        return 0; 
    }

    2
}

fn charger_dictionnaire(chemin: &str) -> Result<HashSet<String>, Box<dyn std::error::Error>> {
    let fichier = File::open(chemin)?;
    let reader = BufReader::new(fichier);
    let mut mots = HashSet::new();

    for ligne in reader.lines() {
        let ligne = ligne?;
        let mot = ligne.trim().to_lowercase();
        if !mot.is_empty() {
            mots.insert(mot);
        }
    }
    Ok(mots)
}

fn charger_donnees(chemin: &str) -> Result<Vec<TweetEtiquete>, Box<dyn std::error::Error>> {
    let fichier = File::open(chemin)?;
    let reader = BufReader::new(fichier);
    let mut donnees = Vec::new();
    
    for (index, ligne) in reader.lines().enumerate() {
        let ligne = ligne?;
        if index == 0 { continue; }
        
        let colonnes: Vec<&str> = ligne.split(',').collect();
        if colonnes.len() >= 2 {
            if let Ok(etiquette) = colonnes[0].parse::<i32>() {
                let contenu_raw = colonnes[1..].join(",");
                let contenu = contenu_raw.trim_matches('"').to_string();
                
                donnees.push(TweetEtiquete {
                    contenu,
                    etiquette,
                });
            }
        }
    }
    Ok(donnees)
}

fn diviser_donnees_stratifiee(donnees: &[TweetEtiquete], ratio_train: f64) -> (Vec<TweetEtiquete>, Vec<TweetEtiquete>) {
    let mut rng = rng();
    let mut par_classe: HashMap<i32, Vec<TweetEtiquete>> = HashMap::new();
    
    // Grouper
    for t in donnees {
        par_classe.entry(t.etiquette).or_insert_with(Vec::new).push(t.clone());
    }
    
    let mut entrainement = Vec::new();
    let mut test = Vec::new();
    
    // Splitter
    for (_, mut vec) in par_classe {
        vec.shuffle(&mut rng);
        let split = (vec.len() as f64 * ratio_train) as usize;
        entrainement.extend_from_slice(&vec[..split]);
        test.extend_from_slice(&vec[split..]);
    }
    
    // Mélanger
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