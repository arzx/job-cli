use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use printpdf::*;
use chrono::Local;

const DATA_FILE: &str = "jobs.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Job {
    id: u32,
    company: String,
    title: String,
    date_submitted: String,
    docs_used: String,
    location: String,
    final_answer: Option<String>,
}

#[derive(Parser)]
#[command(name = "job-tracker")]
#[command(about = "A CLI tool to track job applications")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new job application
    Add {
        company: String,
        title: String,
        docs: String,
        location: String,
        date: Option<String>,
    },
    /// Update a job application (specifically final answer)
    Update {
        #[arg(long)]
        id: u32,
        #[arg(short, long)]
        answer: String,
    },
    /// Delete a job application
    Delete {
        #[arg(long)]
        id: u32,
    },
    /// List all job applications
    List,
    /// Export jobs to a PDF file
    Export {
        #[arg(short, long, default_value = "jobs.pdf")]
        output: String,
    },
    /// Import jobs from a CSV file
    Import {
        file: String,
    },
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    let mut jobs = load_jobs()?;

    match cli.command {
        Commands::Add {
            company,
            title,
            docs,
            location,
            date,
        } => {
            let date_submitted = date.unwrap_or_else(|| Local::now().format("%Y-%m-%d").to_string());
            let id = jobs.iter().map(|j| j.id).max().unwrap_or(0) + 1;
            let job = Job {
                id,
                company,
                title,
                date_submitted,
                docs_used: docs,
                location,
                final_answer: None,
            };
            jobs.push(job.clone());
            save_jobs(&jobs)?;
            println!("Added job: {} at {} (ID: {})", job.title, job.company, job.id);
        }
        Commands::Update { id, answer } => {
            if let Some(job) = jobs.iter_mut().find(|j| j.id == id) {
                job.final_answer = Some(answer.clone());
                save_jobs(&jobs)?;
                println!("Updated job {} with final answer: {}", id, answer);
            } else {
                println!("Job with ID {} not found.", id);
            }
        }
        Commands::Delete { id } => {
            let initial_len = jobs.len();
            jobs.retain(|j| j.id != id);
            if jobs.len() < initial_len {
                save_jobs(&jobs)?;
                println!("Deleted job: ID {}", id);
            } else {
                println!("Job with ID {} not found.", id);
            }
        }
        Commands::List => {
            if jobs.is_empty() {
                println!("No jobs tracked yet.");
            } else {
                println!("{:<4} | {:<20} | {:<20} | {:<12} | {:<15} | {:<20} | {:<15}", 
                    "ID", "Company", "Title", "Date", "Location", "Docs", "Answer");
                println!("{}", "-".repeat(115));
                for job in jobs {
                    let answer = job.final_answer.clone().unwrap_or_else(|| "Pending".to_string());
                    println!("{:<4} | {:<20} | {:<20} | {:<12} | {:<15} | {:<20} | {:<15}", 
                        job.id, 
                        truncate(&job.company, 20), 
                        truncate(&job.title, 20), 
                        job.date_submitted, 
                        truncate(&job.location, 15),
                        truncate(&job.docs_used, 20),
                        answer
                    );
                }
            }
        }
        Commands::Export { output } => {
            if let Err(e) = export_to_pdf(&jobs, &output) {
                eprintln!("Failed to export PDF: {}", e);
            } else {
                println!("Exported {} jobs to {}", jobs.len(), output);
            }
        }
        Commands::Import { file } => {
            if let Err(e) = import_from_csv(file, &mut jobs) {
                eprintln!("Failed to import CSV: {}", e);
            } else {
                save_jobs(&jobs)?;
                println!("Imported jobs successfully.");
            }
        }
    }

    Ok(())
}

fn load_jobs() -> io::Result<Vec<Job>> {
    if !Path::new(DATA_FILE).exists() {
        return Ok(Vec::new());
    }
    let file = File::open(DATA_FILE)?;
    let reader = io::BufReader::new(file);
    match serde_json::from_reader(reader) {
        Ok(jobs) => Ok(jobs),
        Err(_) => Ok(Vec::new()), 
    }
}

fn save_jobs(jobs: &[Job]) -> io::Result<()> {
    let json = serde_json::to_string_pretty(jobs)?;
    let mut file = File::create(DATA_FILE)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

fn truncate(s: &str, max_width: usize) -> String {
    if s.len() > max_width {
        format!("{}...", &s[0..max_width-3])
    } else {
        s.to_string()
    }
}

fn import_from_csv(path: String, jobs: &mut Vec<Job>) -> Result<(), Box<dyn std::error::Error>> {
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b';')
        .from_path(path)?;
    
    let mut added_count = 0;
    
    for result in rdr.records() {
        let record = result?;
        // Expected header roughly: Company;Job Title;Date Submitted;Documents Used;Answer;Ort;Number
        // Indices: 0: Company, 1: Job Title, 2: Date Submitted, 3: Documents Used, 4: Answer, 5: Ort
        if record.len() < 6 { continue; }

        let company = record[0].trim().to_string();
        let title = record[1].trim().to_string();
        let date_submitted = record[2].trim().to_string();
        let docs_used = record[3].trim().to_string();
        let answer_raw = record[4].trim().to_string();
        let location = record[5].trim().to_string();
        
        let final_answer = if answer_raw.is_empty() { None } else { Some(answer_raw) };

        // Check for duplicates (company + title)
        if jobs.iter().any(|j| j.company.eq_ignore_ascii_case(&company) && j.title.eq_ignore_ascii_case(&title)) {
            continue;
        }
        
        let id = jobs.iter().map(|j| j.id).max().unwrap_or(0) + 1;
        let job = Job {
            id,
            company,
            title,
            date_submitted,
            docs_used,
            location,
            final_answer,
        };
        jobs.push(job);
        added_count += 1;
    }
    
    println!("Imported {} new jobs.", added_count);
    Ok(())
}
fn export_to_pdf(jobs: &[Job], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (doc, page1, layer1) = PdfDocument::new("Job Applications", Mm(297.0), Mm(210.0), "Layer 1");
    let font_path = "/System/Library/Fonts/Supplemental/Arial.ttf";
    let font = doc.add_external_font(File::open(font_path)?)?;
    
    let mut current_layer = doc.get_page(page1).get_layer(layer1);
    let mut y = 190.0;
    let line_height = 6.0; // Reduced line height slightly for better fit

    // Initial Header
    draw_header(&current_layer, &font, y);
    y -= 10.0;

    for job in jobs {
        if y < 20.0 {
            let (page, layer) = doc.add_page(Mm(297.0), Mm(210.0), "Layer 1");
            current_layer = doc.get_page(page).get_layer(layer);
            y = 190.0;
            draw_header(&current_layer, &font, y);
            y -= 10.0;
        }
        
        // Truncate strings to avoid overlap
        let company = truncate(&job.company, 25);
        let title = truncate(&job.title, 25);
        let _location = truncate(&job.location, 15);
        let answer = truncate(&job.final_answer.clone().unwrap_or_else(|| "Pending".to_string()), 20);
        
        current_layer.use_text(job.id.to_string(), 10.0, Mm(10.0), Mm(y), &font);
        current_layer.use_text(company, 10.0, Mm(30.0), Mm(y), &font);
        current_layer.use_text(title, 10.0, Mm(80.0), Mm(y), &font);
        current_layer.use_text(&job.date_submitted, 10.0, Mm(130.0), Mm(y), &font);
        current_layer.use_text(answer, 10.0, Mm(160.0), Mm(y), &font);

        y -= line_height;
    }

    // --- Statistics Table ---
    y -= 10.0; // Space before stats

    // Calculate stats
    let total_jobs = jobs.len();
    let pending_count = jobs.iter().filter(|j| j.final_answer.is_none()).count();
    let mut status_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    
    for job in jobs {
        if let Some(ans) = &job.final_answer {
            *status_counts.entry(ans.clone()).or_insert(0) += 1;
        }
    }

    // Check space for stats header + at least a few rows
    if y < 40.0 {
        let (page, layer) = doc.add_page(Mm(297.0), Mm(210.0), "Layer 1");
        current_layer = doc.get_page(page).get_layer(layer);
        y = 190.0;
    }

    // Draw Stats Header
    current_layer.use_text("Statistics", 14.0, Mm(10.0), Mm(y), &font);
    y -= 8.0;

    // Draw Total
    current_layer.use_text("Total Applications", 12.0, Mm(10.0), Mm(y), &font);
    current_layer.use_text(total_jobs.to_string(), 12.0, Mm(60.0), Mm(y), &font);
    y -= line_height;

    // Draw Pending
    current_layer.use_text("Pending", 12.0, Mm(10.0), Mm(y), &font);
    current_layer.use_text(pending_count.to_string(), 12.0, Mm(60.0), Mm(y), &font);
    y -= line_height;

    // Draw other statuses
    for (status, count) in status_counts {
        if y < 20.0 {
            let (page, layer) = doc.add_page(Mm(297.0), Mm(210.0), "Layer 1");
            current_layer = doc.get_page(page).get_layer(layer);
            y = 190.0;
        }
        current_layer.use_text(status, 12.0, Mm(10.0), Mm(y), &font);
        current_layer.use_text(count.to_string(), 12.0, Mm(60.0), Mm(y), &font);
        y -= line_height;
    }

    doc.save(&mut std::io::BufWriter::new(File::create(output_path)?))?;
    Ok(())
}

fn draw_header(layer: &PdfLayerReference, font: &IndirectFontRef, y: f64) {
    layer.use_text("ID", 12.0, Mm(10.0), Mm(y), font);
    layer.use_text("Company", 12.0, Mm(30.0), Mm(y), font);
    layer.use_text("Title", 12.0, Mm(80.0), Mm(y), font);
    layer.use_text("Date", 12.0, Mm(130.0), Mm(y), font);
    layer.use_text("Status", 12.0, Mm(160.0), Mm(y), font);
}
