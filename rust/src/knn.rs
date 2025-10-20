use std::collections::{HashSet, HashMap};
use godot::prelude::*;
use std::fs::File;
use std::io::{BufReader, BufRead};
use rand::seq::SliceRandom;
use rand::rng;

#[derive(GodotClass)]
#[class(base=Node)]
struct Knn {
    base: Base<Node>,
}

#[godot_api]
impl INode for Knn {
    fn init(base: Base<Node>) -> Self {
        Self { base }
    }
}

#[derive(Debug, Clone)]
struct TweetEtiquete {
    contenu: String,
    etiquette: i32, // 4=positif, 2=neutre, 0=négatif
}

#[derive(Debug, Clone, Copy)]
enum TypeVote {
    Majoritaire,
    Pondere,
}

#[godot_api]
impl Knn {
    #[func]
    fn knn_execute(&mut self, path: GString, tweet_a_classifier: GString, k: i64, type_vote: i64) -> GString {
        let path_str = path.to_string();
        let tweet_str = tweet_a_classifier.to_string();
        let type_vote_usize = type_vote as usize;
        let k_usize = k as usize;
        
        // Déterminer le type de vote
        let vote_type = match type_vote_usize {
            1 => TypeVote::Pondere,
            _ => TypeVote::Majoritaire, // Par défaut
        };
        
        // Charger les données depuis le CSV
        let base = match charger_donnees(&path_str) {
            Ok(donnees) => donnees,
            Err(e) => {
                self.signals().log_sent().emit(&GString::from(format!("Erreur chargement données: {}", e)));
                return GString::from("ERREUR");
            }
        };
        
        match classifier_tweet(&tweet_str, k_usize, &base, vote_type) {
            Some(classe) => {
                let resultat = match classe {
                    4 => "POSITIF",
                    2 => "NEUTRE",
                    0 => "NÉGATIF",
                    _ => "INCONNU"
                };
                GString::from(resultat)
            }
            None => {
                self.signals().log_sent().emit(&GString::from("Impossible de classifier le tweet"));
                GString::from("ERREUR")
            }
        }
    }

    #[func]
    fn knn_evaluate(&mut self, path: GString, k: i64, type_vote: i64) -> GString {
        let path_str = path.to_string();
        let k_usize = k as usize;
        let type_vote_usize = type_vote as usize;
        
        // Déterminer le type de vote
        let vote_type = match type_vote_usize {
            1 => TypeVote::Pondere,
            _ => TypeVote::Majoritaire,
        };

        // Charger les données depuis le CSV
        let base_complete = match charger_donnees(&path_str) {
            Ok(donnees) => donnees,
            Err(e) => {
                self.signals().log_sent().emit(&GString::from(format!("Erreur chargement données: {}", e)));
                return GString::from("ERREUR");
            }
        };

        // Division stratifiée 2/3 - 1/3
        let (base_entrainement, base_test) = diviser_donnees_stratifiee(&base_complete, 2.0/3.0);

        if base_entrainement.is_empty() || base_test.is_empty() {
            self.signals().log_sent().emit(&GString::from("Base d'entraînement ou test vide après division"));
            return GString::from("ERREUR");
        }

        // Matrice de confusion : [réel][estimé]
        // Indices: 0=négatif, 2=neutre, 4=positif
        let mut matrice_confusion = [[0; 3]; 3];
        let index_map: HashMap<i32, usize> = [(0, 0), (2, 1), (4, 2)].iter().cloned().collect();

        // Évaluation sur le jeu de test
        for tweet_test in &base_test {
            if let Some(classe_estimee) = classifier_tweet(&tweet_test.contenu, k_usize, &base_entrainement, vote_type) {
                if let (Some(&idx_reel), Some(&idx_estime)) = (
                    index_map.get(&tweet_test.etiquette),
                    index_map.get(&classe_estimee)
                ) {
                    matrice_confusion[idx_reel][idx_estime] += 1;
                }
            }
        }

        // Construction du tableau de résultats
        let resultat = format_matrice_confusion(&matrice_confusion);
        GString::from(resultat)
    }
    
    #[signal]
    fn log_sent(message: GString);
}

fn classifier_tweet(x: &str, k: usize, base: &[TweetEtiquete], type_vote: TypeVote) -> Option<i32> {
    if base.is_empty() || k == 0 || k > base.len() {
        return None;
    }
    
    // Étape 1: Initialiser la liste des proches voisins
    let mut proches_voisins: Vec<(f64, i32)> = Vec::new(); // (distance, etiquette)
    
    // Étape 2: Parcourir tous les tweets de la base
    for tweet in base {
        let distance = distance_tweets(x, &tweet.contenu);
        
        if proches_voisins.len() < k {
            proches_voisins.push((distance, tweet.etiquette));
            // Trier par distance croissante
            proches_voisins.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        } else {
            // Vérifier si ce tweet est plus proche que le plus éloigné actuel
            if distance < proches_voisins.last().unwrap().0 {
                // Remplacer le voisin le plus éloigné
                proches_voisins.pop();
                proches_voisins.push((distance, tweet.etiquette));
                proches_voisins.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            }
        }
    }
    
    // Étape 4: Appliquer le type de vote choisi
    match type_vote {
        TypeVote::Majoritaire => vote_majoritaire(&proches_voisins),
        TypeVote::Pondere => vote_pondere(&proches_voisins),
    }
}

/// Vote majoritaire simple (comptage des occurrences)
fn vote_majoritaire(proches_voisins: &[(f64, i32)]) -> Option<i32> {
    let mut votes: HashMap<i32, usize> = HashMap::new();
    
    for (_, etiquette) in proches_voisins {
        *votes.entry(*etiquette).or_insert(0) += 1;
    }
    
    votes
        .into_iter()
        .max_by_key(|&(_, count)| count)
        .map(|(classe, _)| classe)
}

/// Vote pondéré par l'inverse de la distance
/// Les voisins plus proches ont plus de poids
fn vote_pondere(proches_voisins: &[(f64, i32)]) -> Option<i32> {
    let mut votes_ponderes: HashMap<i32, f64> = HashMap::new();
    
    for (distance, etiquette) in proches_voisins {
        // Éviter la division par zéro en ajoutant un petit epsilon
        let poids = if *distance == 0.0 {
            1.0 // Poids maximum pour distance nulle
        } else {
            1.0 / distance // Inverse de la distance
        };
        
        *votes_ponderes.entry(*etiquette).or_insert(0.0) += poids;
    }
    
    votes_ponderes
        .into_iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .map(|(classe, _)| classe)
}

fn charger_donnees(chemin: &str) -> Result<Vec<TweetEtiquete>, Box<dyn std::error::Error>> {
    let fichier = File::open(chemin)?;
    let reader = BufReader::new(fichier);
    let mut donnees = Vec::new();
    
    for (index, ligne) in reader.lines().enumerate() {
        let ligne = ligne?;
        // Ignorer l'en-tête
        if index == 0 {
            continue;
        }
        
        let colonnes: Vec<&str> = ligne.split(',').collect();
        if colonnes.len() >= 2 {
            if let Ok(etiquette) = colonnes[0].parse::<i32>() {
                let contenu = colonnes[1..].join(","); // Gérer les tweets avec des virgules
                donnees.push(TweetEtiquete {
                    contenu,
                    etiquette,
                });
            }
        }
    }
    Ok(donnees)
}

/// Calcule la distance entre deux tweets selon la formule :
/// D(t1, t2) = (Nombre total de mots - Nombre de mots communs) / Nombre total de mots
fn distance_tweets(tweet1: &str, tweet2: &str) -> f64 {
    // Tokeniser les tweets en mots (en minuscules pour ignorer la casse)
    let mots_tweet1: HashSet<String> = tokeniser_tweet(tweet1);
    let mots_tweet2: HashSet<String> = tokeniser_tweet(tweet2);
    
    // Calculer l'union des mots (nombre total de mots distincts)
    let union_mots: HashSet<&String> = mots_tweet1.union(&mots_tweet2).collect();
    let nombre_total_mots = union_mots.len() as f64;
    
    // Calculer l'intersection des mots (mots communs)
    let intersection_mots: HashSet<&String> = mots_tweet1.intersection(&mots_tweet2).collect();
    let nombre_mots_communs = intersection_mots.len() as f64;
    
    // Appliquer la formule de distance
    if nombre_total_mots == 0.0 {
        0.0 // Si les deux tweets sont vides, distance = 0
    } else {
        (nombre_total_mots - nombre_mots_communs) / nombre_total_mots
    }
}

/// Tokenise un tweet en mots (minuscules)
fn tokeniser_tweet(tweet: &str) -> HashSet<String> {
    tweet
        .split_whitespace()
        .map(|mot| mot.to_lowercase())
        .collect()
}

/// Division stratifiée des données (mêmes proportions de classes dans train/test)
fn diviser_donnees_stratifiee(donnees: &[TweetEtiquete], ratio_train: f64) -> (Vec<TweetEtiquete>, Vec<TweetEtiquete>) {
    let mut rng = rng();
    
    // Séparer les données par classe
    let mut par_classe: HashMap<i32, Vec<TweetEtiquete>> = HashMap::new();
    for tweet in donnees {
        par_classe.entry(tweet.etiquette)
            .or_insert_with(Vec::new)
            .push(tweet.clone());
    }

    let mut entrainement = Vec::new();
    let mut test = Vec::new();

    // Pour chaque classe, diviser selon le ratio
    for (_, mut tweets_classe) in par_classe {
        tweets_classe.shuffle(&mut rng);
        let index_split = (tweets_classe.len() as f64 * ratio_train) as usize;
        
        entrainement.extend_from_slice(&tweets_classe[..index_split]);
        test.extend_from_slice(&tweets_classe[index_split..]);
    }

    // Mélanger les ensembles finaux
    entrainement.shuffle(&mut rng);
    test.shuffle(&mut rng);

    (entrainement, test)
}

/// Formate la matrice de confusion selon le format demandé
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