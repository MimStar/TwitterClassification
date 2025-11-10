use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufRead};

use kodama::{Method, linkage};
use godot::prelude::*;

#[derive(Debug, Clone)]
struct Tweet {
    contenu: String,
    mots: HashSet<String>,
    label: i32,          // 4=positif, 2=neutre, 0=négatif
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
    fn hierarchical_execute(&mut self, path: GString, k: i64, method: GString) -> GString {
        let res = run_pipeline(&path.to_string(), k as usize, &method.to_string());
        GString::from(res.unwrap_or_else(|e| format!("Error: {}", e)))
    }
}

fn run_pipeline(csv_path: &str, k: usize, method: &str) -> Result<String, Box<dyn Error>> {
    // 1. Load annotated tweets
    let tweets = charger_tweets_annotes(csv_path)?;

    // 2. Build condensed distance matrix for hierarchical clustering
    let n = tweets.len();
    let mut condensed = Vec::with_capacity(n * (n - 1) / 2);
    
    for i in 0..n - 1 {
        for j in i + 1..n {
            condensed.push(distance(&tweets[i], &tweets[j]));
        }
    }

    // 3. Choose linkage method
    let linkage_method = match method.to_lowercase().as_str() {
        "complete" => Method::Complete,
        "average" => Method::Average,
        "ward" => Method::Ward,
        _ => Method::Average, // default
    };

    // 4. Perform hierarchical clustering
    let dendrogram = linkage(&mut condensed, n, linkage_method);

    // 5. Generate dendrogram visualization
    let visualization = generate_dendrogram_visualization(&dendrogram, &tweets, k);

    Ok(visualization)
}

fn charger_tweets_annotes(chemin: &str) -> Result<Vec<Tweet>, Box<dyn Error>> {
    let fichier = File::open(chemin)?;
    let reader = BufReader::new(fichier);
    let mut donnees = Vec::new();

    for (index, ligne) in reader.lines().enumerate() {
        let ligne = ligne?;
        if index == 0 {
            continue; // skip header
        }
        let colonnes: Vec<&str> = ligne.split(',').collect();
        if colonnes.len() >= 2 {
            if let Ok(etiquette) = colonnes[0].parse::<i32>() {
                let contenu = colonnes[1..].join(","); // handle commas in tweet text
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

/// Distance entre deux tweets (similarité de Jaccard)
fn distance(t1: &Tweet, t2: &Tweet) -> f64 {
    let len1 = t1.mots.len() as f64;
    let len2 = t2.mots.len() as f64;
    let common = t1.mots.intersection(&t2.mots).count() as f64;
    let union = len1 + len2 - common;
    
    if union == 0.0 {
        0.0
    } else {
        (union - common) / union // Distance de Jaccard
    }
}

/// Génère une visualisation textuelle du dendrogramme
fn generate_dendrogram_visualization(
    dendrogram: &kodama::Dendrogram<f64>, 
    tweets: &[Tweet],
    k: usize
) -> String {
    let n = tweets.len();
    
    // 1. Header avec informations générales
    let mut output = String::new();
    output.push_str(&format!("DENDROGRAMME HIÉRARCHIQUE\n"));
    output.push_str(&format!("========================\n"));
    output.push_str(&format!("Nombre d'observations: {}\n", n));
    output.push_str(&format!("Nombre de clusters demandés: {}\n", k));
    output.push_str(&format!("Hauteur maximale du dendrogramme: {:.4}\n\n", dendrogram.steps().last().map(|s| s.dissimilarity).unwrap_or(0.0)));
    
    // 2. Tableau des étapes de fusion
    output.push_str("ÉTAPES DE FUSION:\n");
    output.push_str("Cluster1 | Cluster2 | Dissimilarité | Taille\n");
    output.push_str("---------|----------|---------------|-------\n");
    
    for (_step_idx, step) in dendrogram.steps().iter().enumerate() {
        output.push_str(&format!(
            "{:8} | {:8} | {:13.4} | {:5}\n",
            step.cluster1, step.cluster2, step.dissimilarity, step.size
        ));
    }
    output.push_str("\n");

    // 3. Assignation des clusters pour k clusters
    let clusters = assign_clusters(dendrogram, k);
    
    output.push_str(&format!("ASSIGNATION DES {} CLUSTERS:\n", k));
    output.push_str("Cluster | Tweets\n");
    output.push_str("--------|-------\n");
    
    let mut cluster_tweets: HashMap<usize, Vec<usize>> = HashMap::new();
    for (tweet_idx, &cluster_id) in clusters.iter().enumerate() {
        cluster_tweets.entry(cluster_id).or_insert_with(Vec::new).push(tweet_idx);
    }
    
    for cluster_id in 0..k {
        if let Some(tweet_indices) = cluster_tweets.get(&cluster_id) {
            let tweet_previews: Vec<String> = tweet_indices.iter()
                .take(3) // Limiter à 3 tweets par cluster pour la lisibilité
                .map(|&idx| {
                    let preview = if tweets[idx].contenu.len() > 30 {
                        format!("{}...", &tweets[idx].contenu[..30])
                    } else {
                        tweets[idx].contenu.clone()
                    };
                    format!("[{}: {}]", idx, preview)
                })
                .collect();
            
            output.push_str(&format!(
                "{:7} | {} {}\n",
                cluster_id,
                tweet_previews.join(", "),
                if tweet_indices.len() > 3 { format!("(+{} autres)", tweet_indices.len() - 3) } else { String::new() }
            ));
        }
    }
    output.push_str("\n");

    // 4. Statistiques par cluster
    output.push_str("STATISTIQUES PAR CLUSTER:\n");
    output.push_str("Cluster | Taille | Sentiment moyen\n");
    output.push_str("--------|--------|----------------\n");
    
    for cluster_id in 0..k {
        if let Some(tweet_indices) = cluster_tweets.get(&cluster_id) {
            let sentiment_moyen: f64 = tweet_indices.iter()
                .map(|&idx| tweets[idx].label as f64)
                .sum::<f64>() / tweet_indices.len() as f64;
            
            let sentiment_desc = match sentiment_moyen {
                x if x >= 3.0 => "Positif",
                x if x >= 1.5 => "Neutre",
                _ => "Négatif"
            };
            
            output.push_str(&format!(
                "{:7} | {:6} | {:.2} ({})\n",
                cluster_id, tweet_indices.len(), sentiment_moyen, sentiment_desc
            ));
        }
    }
    // 5. Représentation ASCII simplifiée du dendrogramme
    output.push_str("\nREPRÉSENTATION VISUELLE SIMPLIFIÉE:\n");
    output.push_str(&generate_ascii_dendrogram(dendrogram));
    
    output
}

/// Assigner les observations aux clusters pour k clusters
fn assign_clusters(dendrogram: &kodama::Dendrogram<f64>, k: usize) -> Vec<usize> {
    let n = dendrogram.len() + 1;
    let mut clusters = vec![0; n];
    
    // Initialiser chaque observation dans son propre cluster
    for i in 0..n {
        clusters[i] = i;
    }
    
    // Fusionner jusqu'à ce qu'il reste k clusters
    for step in dendrogram.steps().iter().take(n - k) {
        let min_cluster = step.cluster1.min(step.cluster2);
        let max_cluster = step.cluster1.max(step.cluster2);
        
        // Mettre à jour tous les éléments du cluster fusionné
        for cluster in &mut clusters {
            if *cluster == max_cluster {
                *cluster = min_cluster;
            }
        }
    }
    
    // Renumérotation des clusters de 0 à k-1
    let unique_clusters: Vec<usize> = clusters.iter().cloned().collect::<HashSet<_>>().into_iter().collect();
    let cluster_map: HashMap<usize, usize> = unique_clusters.iter().enumerate()
        .map(|(new_id, &old_id)| (old_id, new_id))
        .collect();
    
    clusters.iter().map(|&c| cluster_map[&c]).collect()
}

/// Génère une représentation ASCII simplifiée du dendrogramme
fn generate_ascii_dendrogram(dendrogram: &kodama::Dendrogram<f64>) -> String {
    let mut output = String::new();
    let steps = dendrogram.steps();
    let max_height = steps.last().map(|s| s.dissimilarity).unwrap_or(1.0);
    
    // Représentation simplifiée sur 20 lignes
    let num_lines = 20;
    for line in 0..num_lines {
        let current_height = max_height * (num_lines - line - 1) as f64 / num_lines as f64;
        let mut line_str = String::new();
        
        for step in steps {
            if step.dissimilarity >= current_height {
                line_str.push_str(" │ ");
            } else {
                line_str.push_str("   ");
            }
        }
        
        // Ajouter une échelle sur le côté
        let scale_height = max_height * (num_lines - line - 1) as f64 / num_lines as f64;
        output.push_str(&format!("{:5.2} {}\n", scale_height, line_str));
    }
    
    // Ligne de base
    output.push_str(&format!("{:5} ", "0.00"));
    for _ in 0..steps.len() {
        output.push_str("───");
    }
    output.push_str("\n");
    
    output
}