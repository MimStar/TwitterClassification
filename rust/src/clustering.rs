use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::fmt::Write;

use rand::seq::SliceRandom;
use rand::rng;

use kodama::{Method, linkage};
use godot::prelude::*;

#[derive(Debug, Clone)]
struct Tweet {
    id: usize,
    contenu: String,
    mots: HashSet<String>,
    label: i32, // 4=positif, 2=neutre, 0=négatif
}

#[derive(GodotClass)]
#[class(base = Node)]
struct Clustering {
    base: Base<Node>,
}

#[godot_api]
impl INode for Clustering {
    fn init(base: Base<Node>) -> Self {
        Self { base }
    }
}

#[godot_api]
impl Clustering {
    #[func]
    fn clustering_evaluate(&mut self, path: GString, k: i64, method: i64) -> Variant {
        let path_str = path.to_string();
        let mut result_dict = Dictionary::new();

        match run_evaluation_pipeline(&path_str, k as usize, method as usize) {
            Ok((svg, matrix_str)) => {
                result_dict.insert("status", "OK");
                result_dict.insert("svg", svg);
                result_dict.insert("matrix", matrix_str);
            },
            Err(e) => {
                godot_print!("Clustering Error: {}", e);
                result_dict.insert("status", "ERROR");
                result_dict.insert("message", format!("Erreur: {}", e));
            }
        }
        result_dict.to_variant()
    }

    #[func]
    fn clustering_execute(&mut self, path: GString, tweet_content: GString, k: i64, method: i64) -> GString {
        let tweet_str = tweet_content.to_string();
        let path_str = path.to_string();
        
        match predict_tweet_class(&path_str, &tweet_str, k as usize, method as usize) {
            Ok(predicted_label) => {
                let text = match predicted_label {
                    4 => "POSITIF",
                    2 => "NEUTRE",
                    0 => "NÉGATIF",
                    _ => "INCONNU"
                };
                GString::from(text)
            },
            Err(e) => {
                godot_print!("Clustering Execute Error: {}", e);
                GString::from("ERREUR")
            }
        }
    }
}

fn run_evaluation_pipeline(csv_path: &str, k: usize, method: usize) -> Result<(String, String), Box<dyn Error>> {
    // Chargement des données
    let tweets = charger_tweets_annotes(csv_path)?;
    let n = tweets.len();
    if n == 0 { return Err("Aucun tweet trouvé.".into()); }
    if n < k { return Err(format!("Pas assez de tweets ({}) pour K={}", n, k).into()); }

    // Division du dataset : 2/3 entraînement, 1/3 test
    let (mut train, test) = diviser_donnees_stratifiee(&tweets, 2.0 / 3.0);

    let n_train = train.len();
    if n_train < k { return Err(format!("Pas assez de tweets d'entrainement ({}) pour K={}", n_train, k).into()); }

    // Ré-indexer les ID du train pour correspondre à leur position dans le vecteur (pour Kodama)
    for (idx, t) in train.iter_mut().enumerate() {
        t.id = idx;
    }
    
    // Calcul de la matrice de distance en comparant chaque tweet avec les autres
    let mut condensed = Vec::with_capacity(n_train * (n_train - 1) / 2);
    for i in 0..n_train - 1 {
        for j in i + 1..n_train {
            condensed.push(distance(&train[i], &train[j]));
        }
    }

    // Clustering hiérarchique
    let linkage_method = match method { 0 => Method::Average, 1 => Method::Complete, 2 => Method::Ward, _ => Method::Average };
    let dendrogram = linkage(&mut condensed, n_train, linkage_method);

    // Génération du SVG
    let svg = generate_dendrogram_svg(&dendrogram, &train);

    // Découpage de l'arbre pour obtenir K clusters avec UnionFind pour fusionner les groupes jusqu'à avoir K groupes
    let steps = dendrogram.steps();
    let max_index = if steps.is_empty() { n_train } else { n_train + steps.len() };
    let mut uf = UnionFind::new(max_index);
    
    let steps_to_process = if n_train > k { n_train - k } else { 0 };
    for step in steps.iter().take(steps_to_process) {
        uf.union(step.cluster1, step.cluster2);
    }

    // Vote majoritaire et Matrice de confusion en regardant quel est le sentiment dominant dans chaque cluster formé.
    let mut cluster_votes: HashMap<usize, HashMap<i32, usize>> = HashMap::new();
    for tweet in &train {
        let root = uf.find(tweet.id);
        *cluster_votes.entry(root).or_default().entry(tweet.label).or_default() += 1;
    }

    // On assigne une étiquette finale à chaque cluster
    let mut cluster_labels: HashMap<usize, i32> = HashMap::new();
    for (root, votes) in cluster_votes {
        let best_label = votes.into_iter().max_by_key(|&(_, count)| count).map(|(l, _)| l).unwrap_or(2);
        cluster_labels.insert(root, best_label);
    }

    // Construction de la matrice de confusion (Réel vs Estimé)
    let mut matrice = [[0; 3]; 3];
    let index_map: HashMap<i32, usize> = [(0, 0), (2, 1), (4, 2)].iter().cloned().collect();

    for t_test in &test {
        // Trouver le voisin le plus proche dans le train
        let mut best_dist = f64::MAX;
        let mut best_neighbor_idx = 0; // ID dans le train

        for t_train in &train {
            let d = distance(t_test, t_train);
            if d < best_dist {
                best_dist = d;
                best_neighbor_idx = t_train.id;
            }
        }

        // Trouver le cluster du voisin
        let root = uf.find(best_neighbor_idx);
        // Prédire le label du cluster
        let label_estime = *cluster_labels.get(&root).unwrap_or(&2);

        // Mettre à jour la matrice
        if let (Some(&idx_reel), Some(&idx_estime)) = (index_map.get(&t_test.label), index_map.get(&label_estime)) {
            matrice[idx_reel][idx_estime] += 1;
        }
    }

    Ok((svg, format_matrice_confusion(&matrice)))
}

fn diviser_donnees_stratifiee(donnees: &[Tweet], ratio_train: f64) -> (Vec<Tweet>, Vec<Tweet>) {
    let mut rng = rng();
    let mut par_classe: HashMap<i32, Vec<Tweet>> = HashMap::new();
    
    // Grouper par classe
    for t in donnees {
        par_classe.entry(t.label).or_insert_with(Vec::new).push(t.clone());
    }
    
    let mut entrainement = Vec::new();
    let mut test = Vec::new();
    
    // Splitter chaque classe
    for (_, mut vec) in par_classe {
        vec.shuffle(&mut rng); // Mélange aléatoire
        let split = (vec.len() as f64 * ratio_train) as usize;
        
        entrainement.extend_from_slice(&vec[..split]);
        test.extend_from_slice(&vec[split..]);
    }
    
    // Mélanger le tout pour ne pas avoir les classes groupées
    entrainement.shuffle(&mut rng);
    test.shuffle(&mut rng);
    
    (entrainement, test)
}

fn predict_tweet_class(path: &str, input_tweet: &str, k: usize, method: usize) -> Result<i32, Box<dyn Error>> {
    // On refait le clustering ici pour avoir le contexte des groupes
    let tweets = charger_tweets_annotes(path)?;
    let n = tweets.len();
    if n < k { return Err("K est trop grand".into()); }

    let mut condensed = Vec::new();
    for i in 0..n - 1 {
        for j in i + 1..n {
            condensed.push(distance(&tweets[i], &tweets[j]));
        }
    }

    let linkage_method = match method { 0 => Method::Average, 1 => Method::Complete, 2 => Method::Ward, _ => Method::Average };
    let dendrogram = linkage(&mut condensed, n, linkage_method);

    // Reconstruction des clusters
    let steps = dendrogram.steps();
    let max_index = if steps.is_empty() { n } else { n + steps.len() };
    let mut uf = UnionFind::new(max_index);

    let steps_to_run = if n > k { n - k } else { 0 };
    for step in steps.iter().take(steps_to_run) {
        uf.union(step.cluster1, step.cluster2);
    }

    // Calcul des labels des clusters via vote
    let mut cluster_votes: HashMap<usize, HashMap<i32, usize>> = HashMap::new();
    for tweet in &tweets {
        let root = uf.find(tweet.id);
        *cluster_votes.entry(root).or_default().entry(tweet.label).or_default() += 1;
    }

    let mut cluster_labels: HashMap<usize, i32> = HashMap::new();
    for (root, votes) in cluster_votes {
        let lbl = votes.into_iter().max_by_key(|&(_, c)| c).map(|(l, _)| l).unwrap_or(2);
        cluster_labels.insert(root, lbl);
    }

    // Voisin le plus proche en cherchant quel tweet connu ressemble le plus au tweet entré par l'utilisateur.
    let input_words = tokeniser_tweet(input_tweet);
    let input_obj = Tweet { id: 0, contenu: input_tweet.to_string(), mots: input_words, label: -1 };
    
    let mut best_dist = f64::MAX;
    let mut best_neighbor_idx = 0;

    for tweet in &tweets {
        let d = distance(&input_obj, tweet);
        if d < best_dist {
            best_dist = d;
            best_neighbor_idx = tweet.id;
        }
    }

    // On retourne le label du cluster auquel appartient ce voisin
    let neighbor_cluster = uf.find(best_neighbor_idx);
    Ok(*cluster_labels.get(&neighbor_cluster).unwrap_or(&2))
}

// Ne fonctionne pas exactement comme prévu dû à Godot qui n'affiche pas le texte. A voir comme un proof of concept
fn generate_dendrogram_svg(dendrogram: &kodama::Dendrogram<f64>, tweets: &[Tweet]) -> String {
    let n = tweets.len();
    let steps = dendrogram.steps();

    let total_nodes = n + steps.len();
    
    let mut children_map: Vec<Option<(usize, usize)>> = vec![None; total_nodes + 1];

    for (i, step) in steps.iter().enumerate() {
        let new_cluster_idx = n + i;
        if new_cluster_idx < children_map.len() {
             children_map[new_cluster_idx] = Some((step.cluster1, step.cluster2));
        }
    }

    let root_idx = if steps.is_empty() { 
        0 
    } else { 
        n + steps.len() - 1 
    };

    let mut visual_order = Vec::with_capacity(n);
    get_leaf_order(root_idx, &children_map, &mut visual_order);

    let width = 1200.0;
    let height = 800.0;
    let margin = 50.0;
    let drawing_height = height - 2.0 * margin;
    let drawing_width = width - 2.0 * margin;
    let max_dissim = steps.last().map(|s| s.dissimilarity).unwrap_or(1.0).max(0.0001);

    let mut leaf_x_positions = HashMap::new();
    let step_x = drawing_width / (n as f64).max(1.0);
    for (vis_idx, &original_idx) in visual_order.iter().enumerate() {
        let x = margin + (vis_idx as f64 * step_x) + (step_x / 2.0);
        leaf_x_positions.insert(original_idx, x);
    }

    let mut svg = String::with_capacity(50 * 1024);

    let _ = write!(&mut svg, r##"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}"><rect width="100%" height="100%" fill="#202020"/><style>.link {{ stroke: #4a90e2; stroke-width: 2; fill: none; }} .text {{ font-family: sans-serif; font-size: 12px; fill: #e0e0e0; }} .grid {{ stroke: #333; stroke-width: 1; stroke-dasharray: 4; }}</style>"##, width, height, width, height);
    
    let _ = write!(&mut svg, r#"<text x="{}" y="30" font-size="20" fill="white" text-anchor="middle">Dendrogramme (n={})</text>"#, width/2.0, n);

    let mut node_pos: HashMap<usize, (f64, f64)> = HashMap::new();
    let base_y = margin + drawing_height;

    for i in 0..n {
        if let Some(&x) = leaf_x_positions.get(&i) {
            node_pos.insert(i, (x, base_y));
            let color = match tweets[i].label { 4 => "#4caf50", 0 => "#f44336", _ => "#9e9e9e" };
            let safe_text = if tweets[i].contenu.len() > 15 { format!("{}...", &tweets[i].contenu[..15]) } else { tweets[i].contenu.clone() }
                .replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;");
            
            let _ = write!(&mut svg, r#"<circle cx="{}" cy="{}" r="4" fill="{}" /><g transform="translate({}, {}) rotate(45)"><text x="5" y="0" class="text" font-size="10">{}</text></g>"#, x, base_y + 10.0, color, x, base_y + 15.0, safe_text);
        }
    }

    for (i, step) in steps.iter().enumerate() {
        let h_ratio = step.dissimilarity / max_dissim;
        let merge_y = base_y - (h_ratio * drawing_height);
        let c1 = step.cluster1; 
        let c2 = step.cluster2;
        let new_cluster = n + i;

        if let (Some(&(x1, y1)), Some(&(x2, y2))) = (node_pos.get(&c1), node_pos.get(&c2)) {
            let new_x = (x1 + x2) / 2.0;
            node_pos.insert(new_cluster, (new_x, merge_y));
            let _ = write!(&mut svg, r#"<path d="M{} {} V{} H{} V{}" class="link" />"#, x1, y1, merge_y, x2, y2);
        }
    }

    let _ = write!(&mut svg, "</svg>");
    svg
}

// Structure pour gérer les ensembles disjoints, pour couper l'arbre
struct UnionFind {
    parent: Vec<usize>,
}

impl UnionFind {

    fn new(size: usize) -> Self {
        Self { parent: (0..size + 1).collect() }
    }
    
    // Trouve la racine d'un élément
    fn find(&mut self, i: usize) -> usize {
        if i >= self.parent.len() {
            return i; 
        }
        if self.parent[i] == i {
            i
        } else {
            let root = self.find(self.parent[i]);
            self.parent[i] = root;
            root
        }
    }

    // Fusionne deux ensembles
    fn union(&mut self, i: usize, j: usize) {
        if i >= self.parent.len() || j >= self.parent.len() { return; }
        let root_i = self.find(i);
        let root_j = self.find(j);
        if root_i != root_j {
            self.parent[root_i] = root_j;
        }
    }
}

// Utilisé dans le SVG pour déterminer l'ordre des feuilles pour éviter de croiser les lignes
fn get_leaf_order(node: usize, children: &[Option<(usize, usize)>], order: &mut Vec<usize>) {
    if node >= children.len() { return; }

    match children[node] {
        Some((left, right)) => {
            get_leaf_order(left, children, order);
            get_leaf_order(right, children, order);
        },
        None => {
            order.push(node);
        }
    }
}

fn charger_tweets_annotes(chemin: &str) -> Result<Vec<Tweet>, Box<dyn Error>> {
    
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
                donnees.push(Tweet {
                    id: index - 1,
                    mots: tokeniser_tweet(&contenu),
                    contenu,
                    label: etiquette,
                });
            }
        }
    }
    Ok(donnees)
}

fn tokeniser_tweet(t: &str) -> HashSet<String> {
    t.split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_ascii_lowercase())
        .collect()
}

fn distance(t1: &Tweet, t2: &Tweet) -> f64 {
    let len1 = t1.mots.len() as f64;
    let len2 = t2.mots.len() as f64;
    let common = t1.mots.intersection(&t2.mots).count() as f64;
    let union = len1 + len2 - common;
    if union == 0.0 { 1.0 } else { (union - common) / union }
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