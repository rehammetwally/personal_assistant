use std::collections::HashMap;
use std::io::{self, Write};

use crate::agent::UserProfile;

fn read_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

pub fn add_expense(expenses: &mut HashMap<String, f64>, profile: &mut UserProfile) {
    let category = read_input("Expense category: ");
    let amount_input = read_input("Amount: $");
    let amount: f64 = amount_input.parse().unwrap_or(0.0);

    if amount <= 0.0 {
        println!("❌ Invalid amount.");
        return;
    }

    *expenses.entry(category.clone()).or_insert(0.0) += amount;
    profile.add_spending(&category, amount);

    println!("💸 Expense added: ${:.2} for '{}'", amount, category);
}

pub fn view_expenses(expenses: &HashMap<String, f64>) {
    if expenses.is_empty() {
        println!("No expenses recorded.");
        return;
    }

    let total: f64 = expenses.values().sum();
    
    println!("\n💰 Your Expenses:");
    println!("{}", "─".repeat(40));
    
    let mut sorted: Vec<_> = expenses.iter().collect();
    sorted.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
    
    for (category, amount) in sorted {
        let percentage = (amount / total) * 100.0;
        println!("  {} : ${:.2} ({:.1}%)", category, amount, percentage);
    }
    
    println!("{}", "─".repeat(40));
    println!("  Total: ${:.2}", total);
}

pub fn budget_advice(profile: &UserProfile) {
    println!("\n📊 Budget Overview");
    println!("{}", "─".repeat(40));
    println!("Total spending: ${:.2}", profile.total_spending);

    if profile.total_spending > 5000.0 {
        println!("⚠️  Warning: High spending detected!");
        println!("   Consider reviewing your expenses.");
    } else if profile.total_spending > 2000.0 {
        println!("📈 Moderate spending. Keep tracking!");
    } else {
        println!("✅ Your spending is under control.");
    }
    println!("{}", "─".repeat(40));
}
