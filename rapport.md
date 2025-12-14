# Sommaire

- [Sommaire](#sommaire)
- [Introduction](#introduction)
  - [Problématique](#problématique)
  - [Architecture](#architecture)
  - [Organisation](#organisation)
- [En profondeur](#en-profondeur)
  - [Données](#données)
    - [regex\_ext](#regex_ext)
    - [csv\_ext](#csv_ext)
    - [clean\_data](#clean_data)
      - [rule\_filter](#rule_filter)
      - [main](#main)
  - [Classification](#classification)
    - [Naive](#naive)
    - [KNN](#knn)
      - [Stratification (optionnel)](#stratification-optionnel)
      - [Distance](#distance)
      - [Vote](#vote)
    - [Clustering](#clustering)
    - [Bayes](#bayes)



# Introduction

## Problématique

Le projet consiste à explorer le développement d'une pipeline de traitement de données en vue de leur classification. Plus spécifiquement, on s'intéresse à associer des émotions (positive, neutre ou négative) à des *tweets*.

Il s'agit alors de constituer une base, la nettoyer pour éviter le bruit et irrégularités impertinentes, puis de l'utiliser dans divers algorithmes de classification et de mesurer leurs résultats.  

## Architecture

Le logiciel suit un modèle classique MVC. La vue & controlleur sont réalisés aux seins du moteur open source Godot, et le modèle est construit en rust.

La glue entre le modèle et la vue sont réalisés en partie en rust, et d'autre en gdscript, un langage interne au moteur Godot.

La surface du modèle exposée à godot est minime -- la glue. La librairie est construite de sorte à ce qu'elle puisse subsister indépendamment de Godot.

Vous retrouverez dans ce projet l'arborescente (abrégée) suivante :

| nom | brève description |
|-----|-------------------|
| *rust/ - backend* | --------------------------- |
| [csv_ext](rust/src/csv_ext/) | Des outils supplémentaires construit pour divers traitements de fichiers CSV. |
| [regex_ext](rust/src/regex_ext/) | Des outils supplémentaire construit pour la gestion des regex. |
| [clean_data](rust/src/cleandata/) | Module de nettoyage de données, notion de règle de filtres, et règles de base. |
| [naive_classification](rust/src/naive_classification.rs) | Module de classification naive. |
| [knn](rust/src/knn.rs) | Implémentation de l'algorithme KNN. |
| [clustering](rust/src/clustering.rs) | Implémentation de l'algorithme de clustering. |
| [bayes](rust/src/bayes.rs) | Implémentation de l'algorithme de bayes. |
| *godot/ - frontend & glue* | --------------------------- |
| [assets](godot/assets/) | Définission du thème de l'application. |
| [scenes](godot/scenes/) | Scènes, éléments GUI, etc. |
| [scripts](godot/scripts/) | Scripts de glue de la vue. |
| *test-rust/ testing* | *un "playground" de test rust.* |
| *[rapport.md](rapport.md)* | *le rapport de ce projet.* |

## Organisation

Rémy - GUIs, knn, clustering, bayes & glue.  
Shems - Nettoyage, naïf, tooling & rapport.

# En profondeur

## Données

### regex_ext

**builder** est un module qui définit un "Logical Regex Builder", un outil créé sur mesure pour rendre la construction de filtre regex plus intuitive et lisible. On évite les grosse chaîne de caractère obscure en plein code, pour une approche fonctionnelle plus rust-idiomatic. 

Par exemple, le filtre détectant les noms d'utilisateur présents dans les tweets ressemble à -
```rs
RegexLogicalBuilder::from("@")
    .plus_non_space()
    .any_times()
    .as_whole_word()
    .build()
```

### csv_ext

Rust est un merveilleux langage, mais python est beaucoup plus communément utilisé pour ce genre de traitement (bien que l'on pourrait probablement bénéficier des performances accrues de rust !).

Ainsi, le nettoyage des données s'est déjà révélé un petit challenge puisque les librairies rust sont bien moins fournies pour le traitement de csv. Il a fallut définir des fonctionnalités sur-mesure pour rendre notre application robuste à de très divers formats d'entrées, puisque le format csv n'a pas de convention de normalisation.

Il s'agit d'être robuste à
- l'encodage des données
- le format des données
- la structure des données

**[encoding.rs](rust/src/csv_ext/encoding.rs)** définit une simple fonction qui permet de détecter l'encodage d'une suite d'octets, basée sur la librairie chardetng.

> Rust encode ses chaînes de caractères en UTF-8, mais un csv peut contenir n'importe quel encodage. Ainsi, nous ne pouvons pas simplement récupérer les données sous forme de `String/&str`. On traite alors les données en octets bruts, et les décode manuellement à la volée.  
> En ce qui concerne twitter, il semble que les messages y soient contraints à UTF-16. Malgré tout, il est plus précautionneux de ne faire aucune assomption.

**[transform.rs](rust/src/csv_ext/transform.rs)** définit des méthodes pour transformer des données CSV en d'autres format de données pour simplifier certaines manipulations.

**[cols_sniffer](rust/src/csv_ext/cols_sniffer/)** est le plus gros morceau - c'est un module de détection statistique de champs. Il nous permet (d'essayer) de trouver dans quelle colonne se trouve les données qui nous intéressent.

Nous ne rentrerons pas ici dans tous les détails puisque ce n'est pas le sujet premier de notre travail, mais il a demandé une attention particulière, en décrire brièvement les challenges et fonctionnements est alors pertinent.

Nous nous servons de la librairie `csv_sniffer` pour détecter la présence ou non de header. Nous avions commencé par le faire par nous-même, mais cela s'est révélé être une tâche statistique très complexe.

- Naivement, on peut vouloir vérifier que la première colonne ne contient que du texte. D'accord, mais qu'est ce que du texte ?
  - Quelque chose d'entouré par des guillemets ? - **Non**, CSV n'a pas de norme, cela n'est donc pas forcément le cas.
  - Quelque chose que rust peut parse en `String` ? **Non**, il existe de nombreux encodages autre qu'UTF-8, en particuliers les bases de données de tests fournies en contiennent d'autres.
  - Il faut alors vérifier que la donnée est encodable pour X encodages choisis arbitrairements.
 
- On peut maintenant déterminer si la première ligne peut-être décodé en texte. Et qu'en fait-on ?
  - Si c'est le cas, c'est un header ? **Non**, en réalité, il y a peu de chance que l'on ne parvienne pas à encoder une donnée en texte. `6` peut autant être considéré comme du texte que `"6"`.
  - Alors on cherche des headers qui ne peuvent être convertit *qu'en* texte ? **Non**, (1) rien n'oblige un header à être strictement textuel.
  - (2) On impose arbitrairement cette contrainte, on détermine alors que c'est un header dans ce cas ? **Non**, rien ne nous permet cette conclusion - le tableur pourrait être entièrement rempli de texte, et cette première ligne pourrait alors être de la simple donnée.
  - (3) Si le tableau est rempli de texte, il n'y a donc pas de header, que de la donnée ? **Non**, il est possible qu'il y ait bien des headers, le déterminer devient probabilistique, selon la relation de taille entre les colonnes du headers et le reste du tableur par exemple ...

En bref, le travail est long et fastidieux, pour un résultat qui de toute manière est statistique, donc incertains.

La crate `csv_sniffer` offre des résultats plutôt concluant pour cette tâche, alors nous nous en contenterons. Elle nous impose en revanche une contrainte importante : cette crate suppose que les données sont protégées par des guillemets, ce qui n'est pas forcément vrai puisque pour rappel, le format csv n'a aucune spécification. Nous accepterons cette contrainte pour notre programme.

En revanche, il est possible qu'il n'y ait pas de ligne header, ou que celle-ci ne nous aide pas à trouver les données qui nous intéressent. Dans ce cas, déterminer ces colonnes devient très spécifique à notre cas d'usage, et aucune crate ne pourra nous sauver !

Nous faisons alors une analyse statistique plutôt naïve sur les x données (par défaut, les 10 premières lignes de données) pour les identifier. Pour les tweets, on cherche une colonne dont la longueur des messages n'excède jamais 280 charactères (~560 bytes), et dont la longueur moyenne s'en rapproche le plus possible.  
Pour le rating, on cherche une colonne dont le contenu n'est jamais autre chose que 0, 2 ou 4, avec d'éventuelles guillemets.

### clean_data

Ce module a pour responsabilité le nettoyage de données csv en vue d'être utilisées par nos divers algorithmes de classification.

La volonté était de le rendre le plus ré-utilisable, extensible et évolutif possible. Ainsi, sa structure est soigneusement pensée.

Comme pour les outils précédents, les erreurs sont exprimées explicitements, idiomatiquement à rust, en particuliers au travers de la très commune librairie `this_error`.

#### rule_filter

On définit un principe de règle de filtrage dans [rule_filter](rust/src/cleandata/rule_filter/).
- Trimming, qui retire le regex matché de la donnée.
- Replace, qui remplace le regex matché par le paramètre donné.
- Delete, qui drop - c'est à dire qu'il supprime complètement la donnée.

```rs
// First parameter is a log to display
// Second is the regex to match
// Others are specific to the filter type 
#[derive(Debug, Clone)]
pub enum RuleFilter {
    TRIM(String, Regex),            // trim matching from entry
    REPLACE(String, Regex, String), // replace matching from entry
    DELETE(String, Regex),          // delete entry if matching
}

impl RuleFilter {
    // ... Voir plus dans rule_filter.rs ...
}
```
<small>Extrait de [rust/src/cleandata/rule_filter.rs](rust/src/cleandata/rule_filter.rs)</small>

La valeur de retour associée à l'application d'un filtre est une `Option` contenant l'éventuelle donnée restante. 

La fonction `apply_with_logs` permet également de construire un journal de filtrage, étant donné un buffer `logs: &mut Option<String>` fournit.

Les règles de filtres peuvent être ordonnées, afin d'être par exemple triée automatiquement dans une liste de filtre qui permettra de s'assurer que certains filtres sont appliqués en premier.

```rs
// tools.rs
impl RuleFilter {
    // ...
    pub(super) fn rank(&self) -> u8 {
        match self {
            RuleFilter::DELETE(_, _) => 0,
            RuleFilter::REPLACE(_, _, _) => 1,
            RuleFilter::TRIM(_, _) => 2,
        }
    }
}

// external.rs
impl Hash for RuleFilter {/* ... */}
impl PartialEq for RuleFilter { /* ... */ }
impl Eq for RuleFilter { /* ... */ }
impl Ord for RuleFilter { /* ... */ }
impl PartialOrd for RuleFilter { /* ... */ }
```
<small>Extrait de [rust/src/cleandata/rule_filter/tools.rs](rust/src/cleandata/rule_filter/tools.rs) et [rust/src/cleandata/rule_filter/external.rs](rust/src/cleandata/rule_filter/external.rs)</small>

Par défaut, une suppression sera prioritaire sur un remplacement, elle-même prioritaire sur un trimming.

Bien que cela n'aie pas été implémenté par soucis de priorité, il serait trivial d'ajouter une interface utilisateur pour la configuration personnalisée de filtre grâce à cette structure.


#### main

Le corps de la fonction de nettoyage consiste en plusieurs étape.  

Le point d'entrée est définit dans [clean_data.rs](rust/src/cleandata.rs) vers `clean_data_body()` dans [body](rust/src/cleandata/entry.rs).  

On y initialise les règles de filtrage automatiques :
- Supprimer les messages contenant des emojis ascii mixées
- Supprimer les retweets
- Trim les noms d'utilisateurs
- Trim les URLs
- Trim la ponctuation

Ensuite, on "sniff" les colones qui nous intéressent - les messages et l'éventuelle colonne de rating si elle existe déjà.

Dans l'état actuel, si aucune colonne de message n'est trouvée, on prend par défaut une colonne arbitraire (ici la seconde), mais on pourrait à l'avenir ajouter une interface intermédiaire pour demander à l'utilisateur de la sélectionner manuellement.  
En ce qui concerne la colonne de rating, si elle n'est pas trouvée, on en crée alors une dans le fichier de sortie. Ce dernier est ainsi composé de deux colonnes, le rating et le message.

Avec ces colonnes, on appelle le corps principal de la fonction de nettoyage.

Celle-ci applique les filtres donnés en paramètres, et construit un fichier temporaire qui sera utilisé par les algorithmes de classification - `"clean_data_temp.csv"`.

Elle suit aussi l'application des filtres, pour fournir un journal d'opération détaillé. A chaque application de filtre, un message est envoyé au client qui peut l'afficher, et à la complétion du nettoyage, ce sont les statistiques globale de filtrage qui lui sont envoyées.

## Classification

### Naive

La classification naïve consiste à classifier les messages en fonctions de la fréquence d'apparation de mots qui ont été manuellement selectionnés et attribués à une polarité, soit positive soit négative.

Le modèle a plusieurs limitation. Déjà, il ne prend pas en compte les relations entre les mots. "Un beau *insérez votre nom d'oiseau préféré*" est évidemment négatif. Pourtant, on dira ici que c'est plutôt neutre, s'il on considère du moins que beau est positif.

Ce qui nous ammène à la seconde limitation - la fiabilité de la sélection. Mais le problème majeure reste l'absence de considération pour le contexte.

Notre implémentation est plutôt simple, et est configurable par un paramètre de poids qui détermine la "rigueur" de la classification, de 0.5 à 1 (inclus). Ce poids correspond à la proportion minimale de mots d'une polarité (parmis les mots polarisés, donc excluant les mots neutres) pour que le message soit associé à la même polarité.

```rs
fn compute_polarity_with_weight(negatives: u32, positives: u32, weight: f32) -> Result<u32, Box<dyn Error>> {
    let f_negatives : f32 = NumCast::from(negatives)
      .ok_or(format!("Cannot cast {negatives} to f32"))?;

    let f_positives : f32 = NumCast::from(positives)
      .ok_or(format!("Cannot cast {positives} to f32"))?;

    let f_total = f_negatives + f_positives;
    
    let pos_ratio = f_positives / f_total;
    if pos_ratio >= weight {
      return Ok(4);
    }
    
    let neg_ratio = f_negatives / f_total;
    if neg_ratio >= weight {
      return Ok(0);
    }

    Ok(2)
}
```
<small>Extrait de [rust/src/naive_classification.rs](rust/src/naive_classification.rs)</small>

A 1, un message doit contenir uniquement des mots d'une polarité pour y être associé, autrement, il est neutre. A 0.5, il suffit qu'il y ait plus de mots positifs que négatifs (et inversement) pour qu'il soit classifié tel quel.

On peut proposer une autre approche ou la proportion n'est pas calculée sur le nombre de mots polarisés, mais le nombre de mots tout court. On préferera en effet peut-être classifier un message qui ne contient qu'un mot positif, et le reste neutre, en tant que message neutre, bien que 100% des mots polarisés soient positifs.

### KNN

Le principe du KNN, pour K-nearest neighbors, consiste à former une base de données de référence dîtes "d'apprentissage", à partir de laquelle on génère une matrice de distance pour chaque donnée à classifier. C'est à dire calculer un coefficient exprimant la ressemblance entre celle-ci et chaque donnée de notre base d'apprentissage.

Cette base d'apprentissage peut soit, idéalement, être constituée manuellement, soit être générée préalablement au travers d'un autre algorithme d'apprentissage, comme notre classification naive par exemple.

A partir de cette matrice de distance, on pourra pour chaque donnée, déterminer quels sont ses $k$ plus proches voisins, et la classifier selon ceux-ci.

Il y a donc plusieurs "modules" à une telle pipeline de KNN.

#### Stratification (optionnel)

Si aucune donnée d'apprentissage explicite n'est donnée, il faut donc d'abord stratifier les données, les diviser en une part qui nous servira d'apprentissage, et l'autre qui sera classifiée.

Dans notre cas, nous utilisons une stratification semi-aléatoire, la donnée est mélangée mais on maintient une proportion de classe égale dans les données de test et d'apprentissage.
La proportion de stratification elle est de 2 tiers d'apprentissage / 1 tiers de test.

Notez, si vous jetez un oeil à la méthode de stratification (`diviser_donnees_stratifiee` dans [rust/src/knn.rs](rust/src/knn.rs)) que nous mélangeons également les données d'entraînement et d'apprentissage après la stratification. En principe, celà n'est pas nécessaire pour le knn puisque chaque donnée de test est indépendante, et l'ordre des données d'apprentissage ne devraient pas non plus avoir d'influence sur le calcul de distance. On le fait plus par uniformisation, puisque cet aléatoire est nécessaire pour d'autres de nos algorithmes.

#### Distance

Un premier module est donc la manière de déterminer la distance entre des données.

Ici, on en propose une unique, se basant sur le ratio de mots communs.

```rs
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
```
<small>Extrait de [rust/src/knn.rs](rust/src/knn.rs)</small>

#### Vote

Une fois que l'on a determiné les $k$ plus proches voisins à partir d'une distance donnée, il faut encore déterminer la manière de classifier une donnée selon cette nouvelle information.

En particuliers, notre implémentation en propose deux, mais sera facilement extensible à davantages de méthodes -
- Pondérée
- Majoritaire

```rs
#[derive(Debug, Clone, Copy)]
enum TypeVote {
    Majoritaire,
    Pondere,
}

impl TypeVote {
    fn vote(&self, proches_voisins: &[(f64, i32)]) -> Option<i32> {
        match self {
            TypeVote::Majoritaire => vote_majoritaire(proches_voisins),
            TypeVote::Pondere => vote_pondere(proches_voisins),
            // Voir les définition vote_pondere et majoritaire dans knn.rs
        }
    }
}

impl From<i64> for TypeVote {
    fn from(value: i64) -> Self {
        match value {
            1 => TypeVote::Pondere,
            _ => TypeVote::Majoritaire,
        }
    }
}
```
<small>Extrait de [rust/src/knn.rs](rust/src/knn.rs)</small>

Un vote majoritaire classifiera une donnée selon la classe la plus présente parmis les k voisins, peu importe la distance.

Un vote pondéré en revanche prendra en compte cette distance. Il est *pondéré* par cette même distance, c'est à dire que les voisins les plus proches ont un poids plus important dans le vote.

### Clustering

### Bayes



