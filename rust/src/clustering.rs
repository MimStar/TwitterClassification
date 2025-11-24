use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::fmt::Write; // Pour une écriture de string performante

use kodama::{Method, linkage};
use godot::prelude::*;

#[derive(Debug, Clone)]
struct Tweet {
    contenu: String,
    mots: HashSet<String>,
    label: i32,
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
    /// Exécute le clustering et retourne le code source SVG du dendrogramme
    #[func]
    fn hierarchical_execute(&mut self, path: GString, k: i64, method: i64) -> GString {
        let res = run_pipeline(&path.to_string(), k as usize, method as usize);
        // En cas d'erreur, on retourne un SVG contenant le texte de l'erreur
        match res {
            Ok(svg) => GString::from(svg),
            Err(e) => GString::from(format!(
                "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 400 50'><text x='10' y='30' fill='red'>Error: {}</text></svg>", 
                e
            ))
        }
    }
}

fn run_pipeline(csv_path: &str, _k: usize, method: usize) -> Result<String, Box<dyn Error>> {
    // 1. Charger les tweets
    let tweets = charger_tweets_annotes(csv_path)?;
    if tweets.is_empty() {
        return Err("Aucun tweet trouvé".into());
    }

    // 2. Matrice de distance
    let n = tweets.len();
    let mut condensed = Vec::with_capacity(n * (n - 1) / 2);
    
    for i in 0..n - 1 {
        for j in i + 1..n {
            condensed.push(distance(&tweets[i], &tweets[j]));
        }
    }

    // 3. Méthode de linkage
    let linkage_method = match method {
        0 => Method::Average,
        1 => Method::Complete,
        2 => Method::Ward,
        _ => Method::Average,
    };

    // 4. Clustering
    let dendrogram = linkage(&mut condensed, n, linkage_method);

    // 5. Génération SVG
    let svg = generate_dendrogram_svg(&dendrogram, &tweets);

    Ok(svg)
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
                let contenu = colonnes[1..].join(",");
                let mots = tokeniser_tweet(&contenu);
                donnees.push(Tweet {
                    contenu,
                    mots,
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
    if union == 0.0 { 0.0 } else { (union - common) / union }
}

/// Génère le code SVG pour le dendrogramme
fn generate_dendrogram_svg(dendrogram: &kodama::Dendrogram<f64>, tweets: &[Tweet]) -> String {
    let n = tweets.len();
    let steps = dendrogram.steps();

    // --- 1. Configuration de la mise en page ---
    let width = 1200.0;
    let height = 800.0;
    let margin = 50.0;
    let drawing_height = height - 2.0 * margin;
    let drawing_width = width - 2.0 * margin;

    // Hauteur max du dendrogramme (dissimilarité max)
    let max_dissim = steps.last().map(|s| s.dissimilarity).unwrap_or(1.0).max(0.0001);

    // --- 2. Réordonnancement des feuilles (Leaf Ordering) ---
    let mut children_map: Vec<Option<(usize, usize)>> = vec![None; 2 * n];
    for (i, step) in steps.iter().enumerate() {
        let parent_idx = n + i;
        children_map[parent_idx] = Some((step.cluster1, step.cluster2));
    }

    let root_idx = 2 * n - 2; 
    let mut visual_order = Vec::new();
    get_leaf_order(root_idx, &children_map, &mut visual_order);

    let mut leaf_x_positions = HashMap::new();
    let step_x = drawing_width / (n as f64).max(1.0);
    
    for (vis_idx, &original_idx) in visual_order.iter().enumerate() {
        let x = margin + (vis_idx as f64 * step_x) + (step_x / 2.0);
        leaf_x_positions.insert(original_idx, x);
    }

    // --- 3. Construction du SVG ---
    let mut svg = String::with_capacity(100 * 1024); 
    
    // CORRECTION ICI : Utilisation de r##" au lieu de r#" pour éviter le conflit avec fill="#..."
    write!(&mut svg, r##"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">
    <rect width="100%" height="100%" fill="#202020"/>
    <style>
        .link {{ stroke: #4a90e2; stroke-width: 2; fill: none; }}
        .text {{ font-family: sans-serif; font-size: 12px; fill: #e0e0e0; }}
        .axis {{ stroke: #666; stroke-width: 1; }}
        .grid {{ stroke: #333; stroke-width: 1; stroke-dasharray: 4; }}
    </style>"##, width, height, width, height).unwrap();

    // Titre
    write!(&mut svg, r#"<text x="{}" y="30" font-size="20" fill="white" text-anchor="middle">Dendrogramme des Tweets (n={})</text>"#, width/2.0, n).unwrap();

    // --- 4. Dessin de la grille et échelle Y ---
    let num_ticks = 5;
    for i in 0..=num_ticks {
        let ratio = i as f64 / num_ticks as f64;
        let y_pos = margin + drawing_height - (ratio * drawing_height);
        let val = ratio * max_dissim;
        
        write!(&mut svg, r#"<line x1="{}" y1="{}" x2="{}" y2="{}" class="grid" />"#, 
            margin, y_pos, width - margin, y_pos).unwrap();
        write!(&mut svg, r#"<text x="{}" y="{}" class="text" text-anchor="end" alignment-baseline="middle">{:.3}</text>"#, 
            margin - 10.0, y_pos, val).unwrap();
    }

    // --- 5. Calcul et dessin des nœuds ---
    let mut node_pos: HashMap<usize, (f64, f64)> = HashMap::new();

    let base_y = margin + drawing_height;
    for i in 0..n {
        if let Some(&x) = leaf_x_positions.get(&i) {
            node_pos.insert(i, (x, base_y));
            
            let label_color = match tweets[i].label {
                4 => "#4caf50", // Positif
                0 => "#f44336", // Négatif
                _ => "#9e9e9e", // Neutre
            };
            
            write!(&mut svg, r#"<circle cx="{}" cy="{}" r="4" fill="{}" />"#, x, base_y + 10.0, label_color).unwrap();
            
            let short_text = if tweets[i].contenu.len() > 15 {
                format!("{}...", &tweets[i].contenu[..15])
            } else {
                tweets[i].contenu.clone()
            };
            
            let safe_text = short_text.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;");
            
            write!(&mut svg, r#"
                <g transform="translate({}, {}) rotate(45)">
                    <text x="5" y="0" class="text" font-size="10">{}</text>
                </g>"#, 
                x, base_y + 15.0, safe_text).unwrap();
        }
    }

    for (i, step) in steps.iter().enumerate() {
        let cluster1 = step.cluster1;
        let cluster2 = step.cluster2;
        let new_cluster = n + i;
        
        let h_ratio = step.dissimilarity / max_dissim;
        let merge_y = base_y - (h_ratio * drawing_height);

        if let (Some(&(x1, y1_child)), Some(&(x2, y2_child))) = (node_pos.get(&cluster1), node_pos.get(&cluster2)) {
            let new_x = (x1 + x2) / 2.0;
            node_pos.insert(new_cluster, (new_x, merge_y));

            write!(&mut svg, r#"<line x1="{}" y1="{}" x2="{}" y2="{}" class="link" />"#, x1, y1_child, x1, merge_y).unwrap();
            write!(&mut svg, r#"<line x1="{}" y1="{}" x2="{}" y2="{}" class="link" />"#, x2, y2_child, x2, merge_y).unwrap();
            write!(&mut svg, r#"<line x1="{}" y1="{}" x2="{}" y2="{}" class="link" />"#, x1, merge_y, x2, merge_y).unwrap();
        }
    }

    write!(&mut svg, "</svg>").unwrap();
    svg
}

/// Fonction récursive pour obtenir l'ordre des feuilles (DFS)
fn get_leaf_order(current_node: usize, children: &[Option<(usize, usize)>], order: &mut Vec<usize>) {
    if let Some((left, right)) = children.get(current_node).and_then(|opt| *opt) {
        // C'est un nœud interne, on descend
        get_leaf_order(left, children, order);
        get_leaf_order(right, children, order);
    } else {
        // C'est une feuille
        order.push(current_node);
    }
}