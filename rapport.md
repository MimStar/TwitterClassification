- [Analyse de sentiments](#analyse-de-sentiments)
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



# Analyse de sentiments

## Problématique

Problématique ..

## Architecture

Vue & controlleur via le moteur open source godot.  

Modèle de donnée et logique en rust.

Une partie du modèle est exposée à godot -- la glue mais la majorité est isolée.


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

## Organisation

Rémy - GUIs, algorithmes de classification & glue.  
Shems - Nettoyage de données, tooling & rapport.

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
 
- Mais.. ce n'est pas terminé. Si c'est bien du texte, rien ne nous garantit encore qu'il s'agisse d'un header. Cela pourrait simplement être un tableur rempli entièrement de texte.

- Si c'est le cas, nous ne sommes pas non plus garantit que le tableur ne contient *PAS* de header. Il faut alors estimer la probabilité qu'il s'agisse d'un header selon les relations de tailles entres les différentes cellules ...

En bref, le travail est long et fastidieux, pour un résultat qui de toute manière est statistique, donc incertains.

La crate `csv_sniffer` offre des résultats plutôt concluant pour cette tâche, alors nous nous en contenterons. 

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

La valeur de retour associée à l'application d'un filtre est une `Option` contenant l'éventuelle donnée restante. 

La fonction `apply_with_logs` permet également de construire un journal de filtrage, étant donné un buffer `logs: &mut Option<String>` fournit.

Les règles de filtres peuvent être ordonnées, afin d'être par exemple triée automatiquement dans une liste de filtre qui permettra de s'assurer que certains filtres sont appliqués en premier.

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

Ensuite, on "sniff" les colomnes qui nous intéressent - les messages et l'éventuelle colonne de rating si elle existe déjà.

Dans l'état actuel, si aucune colonne de message n'est trouvée, on prend par défaut une colonne arbitraire (ici la seconde), mais on pourrait à l'avenir ajouter une interface intermédiaire pour demander à l'utilisateur de la sélectionner manuellement.  
En ce qui concerne la colonne de rating, si elle n'est pas trouvée, on en crée alors une dans le fichier de sortie. Ce dernier est ainsi composé de deux colonnes, le rating et le message.

Avec ces colonnes, on appelle le corps principal de la fonction de nettoyage.

Celle-ci applique les filtres donnés en paramètres, et construit un fichier temporaire qui sera utilisé par les algorithmes de classification - `"clean_data_temp.csv"`.

Elle suit aussi l'application des filtres, pour fournir un journal d'opération détaillé. A chaque application de filtre, un message est envoyé au client qui peut l'afficher, et à la complétion du nettoyage, ce sont les statistiques globale de filtrage qui lui sont envoyées.

## Classification






