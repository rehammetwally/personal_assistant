use std::fs::{read_to_string, write};

use crate::tasks::Task;

pub fn save_tasks(tasks: &Vec<Task>) {
    let mut data = String::new();

    for task in tasks {
        data.push_str(&format!("{},{},{}\n", task.id, task.title, task.completed));
    }

    let _ = write("tasks.txt", data);
}

pub fn load_tasks() -> Vec<Task> {
    let content = read_to_string("tasks.txt");
    let mut tasks = Vec::new();

    if let Ok(data) = content {
        for line in data.lines() {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() == 3 {
                tasks.push(Task {
                    id: parts[0].parse().unwrap_or(0),
                    title: parts[1].to_string(),
                    completed: parts[2].parse().unwrap_or(false),
                });
            }
        }
    }

    tasks
}
