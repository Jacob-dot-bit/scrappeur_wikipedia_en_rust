use reqwest;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io::{self, Write};
use clap::Parser;

#[derive(Debug, Serialize, Deserialize)]
struct WikipediaPage {
    title: String,
    summary: String,
    sections: Vec<String>,
    links: Vec<String>,
    images: Vec<String>,
}

#[derive(Parser, Debug)]
#[command(name = "Wikipedia Scraper")]
#[command(about = "Scrape des pages Wikipedia en français", long_about = None)]
struct Args {
    /// Sujet à rechercher sur Wikipedia (ex: "Rust_(langage)" ou "Intelligence_artificielle")
    #[arg(short, long)]
    sujet: Option<String>,

    /// URL complète de la page Wikipedia à scraper
    #[arg(short, long)]
    url: Option<String>,

    /// Mode interactif : demande le sujet à l'utilisateur
    #[arg(short, long)]
    interactif: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    
    let url = if args.interactif {
        // Mode interactif
        get_user_input()?
    } else if let Some(url_complete) = args.url {
        // URL complète fournie
        url_complete
    } else if let Some(sujet) = args.sujet {
        // Sujet fourni, construction de l'URL
        format!("https://fr.wikipedia.org/wiki/{}", sujet)
    } else {
        // Aucun argument : mode interactif par défaut
        get_user_input()?
    };
    
    println!("Scraping de la page Wikipedia : {}", url);
    
    let page_data = scrape_wikipedia(&url).await?;
    
    println!("\n=== Résultats du Scraping ===\n");
    println!("Titre: {}", page_data.title);
    println!("\nRésumé:\n{}", page_data.summary);
    println!("\nNombre de sections: {}", page_data.sections.len());
    println!("Nombre de liens: {}", page_data.links.len());
    println!("Nombre d'images: {}", page_data.images.len());
    
    // Afficher quelques sections
    println!("\nPremières sections:");
    for (i, section) in page_data.sections.iter().take(5).enumerate() {
        println!("  {}. {}", i + 1, section);
    }
    
    // Sauvegarder en JSON
    let filename = format!("{}.json", sanitize_filename(&page_data.title));
    let json = serde_json::to_string_pretty(&page_data)?;
    std::fs::write(&filename, json)?;
    println!("\nDonnées sauvegardées dans '{}'", filename);
    
    Ok(())
}

fn get_user_input() -> Result<String, Box<dyn Error>> {
    println!("\n=== Scraper Wikipedia ===\n");
    println!("Choisissez une option :");
    println!("1. Entrer un sujet (ex: Rust_(langage))");
    println!("2. Entrer une URL complète");
    println!("3. Rechercher par mot-clé");
    
    print!("\nVotre choix (1-3) : ");
    io::stdout().flush()?;
    
    let mut choix = String::new();
    io::stdin().read_line(&mut choix)?;
    
    match choix.trim() {
        "1" => {
            print!("Entrez le sujet (ex: Python_(langage)) : ");
            io::stdout().flush()?;
            let mut sujet = String::new();
            io::stdin().read_line(&mut sujet)?;
            Ok(format!("https://fr.wikipedia.org/wiki/{}", sujet.trim()))
        }
        "2" => {
            print!("Entrez l'URL complète : ");
            io::stdout().flush()?;
            let mut url = String::new();
            io::stdin().read_line(&mut url)?;
            Ok(url.trim().to_string())
        }
        "3" => {
            print!("Entrez un mot-clé à rechercher : ");
            io::stdout().flush()?;
            let mut mot_cle = String::new();
            io::stdin().read_line(&mut mot_cle)?;
            let mot_cle_formate = mot_cle.trim().replace(" ", "_");
            println!("\nRecherche de : {}", mot_cle_formate);
            Ok(format!("https://fr.wikipedia.org/wiki/{}", mot_cle_formate))
        }
        _ => {
            println!("Choix invalide, utilisation de 'Rust_(langage)' par défaut");
            Ok("https://fr.wikipedia.org/wiki/Rust_(langage)".to_string())
        }
    }
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect::<String>()
        .chars()
        .take(50)
        .collect()
}

async fn scrape_wikipedia(url: &str) -> Result<WikipediaPage, Box<dyn Error>> {
    // Récupérer le contenu HTML
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()?;
    
    let response = client.get(url).send().await?;
    let html_content = response.text().await?;
    
    // Parser le HTML
    let document = Html::parse_document(&html_content);
    
    // Extraire le titre
    let title_selector = Selector::parse("h1#firstHeading").unwrap();
    let title = document
        .select(&title_selector)
        .next()
        .map(|el| el.text().collect::<String>())
        .unwrap_or_default();
    
    // Extraire le résumé (premier paragraphe)
    let summary_selector = Selector::parse("div.mw-parser-output > p").unwrap();
    let summary = document
        .select(&summary_selector)
        .find(|el| !el.text().collect::<String>().trim().is_empty())
        .map(|el| el.text().collect::<String>())
        .unwrap_or_default();
    
    // Extraire les sections
    let section_selector = Selector::parse("h2 .mw-headline, h3 .mw-headline").unwrap();
    let sections: Vec<String> = document
        .select(&section_selector)
        .map(|el| el.text().collect::<String>())
        .collect();
    
    // Extraire les liens internes
    let link_selector = Selector::parse("div#mw-content-text a[href^='/wiki/']").unwrap();
    let links: Vec<String> = document
        .select(&link_selector)
        .filter_map(|el| el.value().attr("href"))
        .filter(|href| !href.contains(":"))
        .take(50)
        .map(|href| format!("https://fr.wikipedia.org{}", href))
        .collect();
    
    // Extraire les images
    let image_selector = Selector::parse("img[src]").unwrap();
    let images: Vec<String> = document
        .select(&image_selector)
        .filter_map(|el| el.value().attr("src"))
        .filter(|src| src.starts_with("//"))
        .take(20)
        .map(|src| format!("https:{}", src))
        .collect();
    
    Ok(WikipediaPage {
        title,
        summary,
        sections,
        links,
        images,
    })
}
