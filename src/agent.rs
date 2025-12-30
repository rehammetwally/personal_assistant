use crate::tasks::Task;
use crate::groq::GroqClient;
use chrono::Local;

#[derive(Debug)]
pub struct UserProfile {
    pub frequent_task: Option<String>,
    pub total_spending: f64,
    pub task_history: Vec<String>,
    pub spending_categories: Vec<(String, f64)>,
}

impl UserProfile {
    pub fn new() -> Self {
        Self {
            frequent_task: None,
            total_spending: 0.0,
            task_history: Vec::new(),
            spending_categories: Vec::new(),
        }
    }

    pub fn add_task_to_history(&mut self, task: &str) {
        self.task_history.push(task.to_string());
        if self.task_history.len() > 20 {
            self.task_history.remove(0);
        }
        self.frequent_task = Some(task.to_string());
    }

    pub fn add_spending(&mut self, category: &str, amount: f64) {
        self.spending_categories.push((category.to_string(), amount));
        self.total_spending += amount;
    }
}

pub fn suggest_action(profile: &UserProfile) {
    if let Some(task) = &profile.frequent_task {
        println!(
            "🤖 Suggestion: You often work on '{}'. Want to continue?",
            task
        );
    } else {
        println!("🤖 No suggestions yet. Add more tasks!");
    }
}

pub async fn ai_suggest_action(
    client: &GroqClient,
    profile: &UserProfile,
    tasks: &[Task],
) -> Result<String, String> {
    let current_time = Local::now().format("%H:%M on %A, %B %d").to_string();
    
    let task_list: String = if tasks.is_empty() {
        "No tasks yet.".to_string()
    } else {
        tasks
            .iter()
            .map(|t| format!("- {} ({})", t.title, if t.completed { "done" } else { "pending" }))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let spending_info = if profile.spending_categories.is_empty() {
        "No expenses tracked yet.".to_string()
    } else {
        format!(
            "Total: ${:.2}\nCategories:\n{}",
            profile.total_spending,
            profile.spending_categories
                .iter()
                .map(|(c, a)| format!("  - {}: ${:.2}", c, a))
                .collect::<Vec<_>>()
                .join("\n")
        )
    };

    let system_prompt = r#"You are a helpful personal productivity assistant. 
Provide brief, actionable suggestions based on the user's tasks and spending.
Keep responses concise (2-3 sentences max). Use emojis sparingly.
Consider the time of day when making suggestions."#;

    let user_prompt = format!(
        "Current time: {}\n\nMy tasks:\n{}\n\nMy spending:\n{}\n\nWhat should I focus on next?",
        current_time, task_list, spending_info
    );

    client.chat_with_system(system_prompt, &user_prompt).await
}

pub async fn ai_analyze_budget(
    client: &GroqClient,
    profile: &UserProfile,
) -> Result<String, String> {
    if profile.spending_categories.is_empty() {
        return Ok("📊 No expenses to analyze yet. Add some expenses first!".to_string());
    }

    let spending_details: String = profile
        .spending_categories
        .iter()
        .map(|(c, a)| format!("- {}: ${:.2}", c, a))
        .collect::<Vec<_>>()
        .join("\n");

    let system_prompt = r#"You are a financial advisor assistant.
Analyze the user's spending and provide:
1. A brief summary of their spending patterns
2. One specific area they could reduce spending
3. One positive observation about their finances
Keep the response under 100 words. Use emojis for visual appeal."#;

    let user_prompt = format!(
        "Total spending: ${:.2}\n\nBreakdown:\n{}",
        profile.total_spending, spending_details
    );

    client.chat_with_system(system_prompt, &user_prompt).await
}

pub async fn ai_prioritize_tasks(
    client: &GroqClient,
    tasks: &[Task],
) -> Result<String, String> {
    let pending_tasks: Vec<_> = tasks.iter().filter(|t| !t.completed).collect();
    
    if pending_tasks.is_empty() {
        return Ok("✅ All tasks completed! Great job!".to_string());
    }

    let task_list: String = pending_tasks
        .iter()
        .map(|t| format!("{}. {}", t.id, t.title))
        .collect::<Vec<_>>()
        .join("\n");

    let current_time = Local::now().format("%H:%M on %A").to_string();

    let system_prompt = r#"You are a productivity expert.
Analyze the tasks and suggest a priority order.
Consider the time of day and task types.
Provide brief reasoning (1 sentence per task).
Format: numbered list with task name and reason."#;

    let user_prompt = format!(
        "Current time: {}\n\nPending tasks:\n{}\n\nSuggest priority order:",
        current_time, task_list
    );

    client.chat_with_system(system_prompt, &user_prompt).await
}

pub async fn ai_chat(
    client: &GroqClient,
    conversation: &mut Vec<crate::groq::Message>,
    user_input: &str,
) -> Result<String, String> {
    // Add system message if this is the first message
    if conversation.is_empty() {
        conversation.push(crate::groq::Message::system(
            "You are a helpful personal assistant. Help the user with productivity, \
             task management, and financial advice. Be concise and friendly."
        ));
    }

    // Add user message
    conversation.push(crate::groq::Message::user(user_input));

    // Get AI response
    let response = client.chat(conversation.clone()).await?;

    // Add assistant response to history
    conversation.push(crate::groq::Message::assistant(&response));

    Ok(response)
}
