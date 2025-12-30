use std::io::{self, Write};
use std::thread;
use std::time::Duration;

use crate::agent::UserProfile;

#[derive(Debug, Clone)]
pub struct Task {
    pub id: u32,
    pub title: String,
    pub completed: bool,
}

fn read_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

pub fn add_task(tasks: &mut Vec<Task>, profile: &mut UserProfile) {
    let title = read_input("Enter task title: ");
    let id = tasks.len() as u32 + 1;

    tasks.push(Task {
        id,
        title: title.clone(),
        completed: false,
    });

    profile.add_task_to_history(&title);

    println!("✅ Task added.");
}

pub fn view_tasks(tasks: &Vec<Task>) {
    if tasks.is_empty() {
        println!("No tasks found.");
        return;
    }

    println!("\n📋 Your Tasks:");
    println!("{}", "─".repeat(40));
    for task in tasks {
        let status = if task.completed { "✅" } else { "⏳" };
        println!("{} [{}] {}", status, task.id, task.title);
    }
    println!("{}", "─".repeat(40));
}

pub fn delete_task(tasks: &mut Vec<Task>) {
    let id_input = read_input("Enter task ID to delete: ");
    let id: u32 = id_input.parse().unwrap_or(0);

    let len_before = tasks.len();
    tasks.retain(|t| t.id != id);
    
    if tasks.len() < len_before {
        println!("🗑️ Task deleted.");
    } else {
        println!("❌ Task not found.");
    }
}

pub fn complete_task(tasks: &mut Vec<Task>) {
    let id_input = read_input("Enter task ID to mark complete: ");
    let id: u32 = id_input.parse().unwrap_or(0);

    if let Some(task) = tasks.iter_mut().find(|t| t.id == id) {
        task.completed = true;
        println!("✅ Task '{}' marked as complete!", task.title);
    } else {
        println!("❌ Task not found.");
    }
}

pub fn set_reminder() {
    let seconds_input = read_input("Reminder after how many seconds? ");
    let seconds: u64 = seconds_input.parse().unwrap_or(0);

    if seconds == 0 {
        println!("❌ Invalid time.");
        return;
    }

    let message = read_input("Reminder message: ");

    thread::spawn(move || {
        thread::sleep(Duration::from_secs(seconds));
        println!("\n⏰ REMINDER: {}", message);
    });

    println!("⏳ Reminder set for {} seconds.", seconds);
}
