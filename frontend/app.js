const API_BASE = '/api';
let authToken = localStorage.getItem('token');
let currentUser = null;
let currentAuthMode = 'login';

// --- Initialization ---
document.addEventListener('DOMContentLoaded', () => {
    if (authToken) {
        verifyToken();
    } else {
        showPage('auth');
    }
});

// --- Navigation ---
function showPage(page) {
    if (page === 'auth') {
        document.getElementById('auth-container').classList.remove('hidden');
        document.getElementById('main-container').classList.add('hidden');
    } else {
        document.getElementById('auth-container').classList.add('hidden');
        document.getElementById('main-container').classList.remove('hidden');
        loadDashboardData();
    }
}

function showSection(sectionId) {
    document.querySelectorAll('.section').forEach(s => s.classList.add('hidden'));
    document.getElementById(`section-${sectionId}`).classList.remove('hidden');
    
    document.querySelectorAll('.nav-item').forEach(btn => btn.classList.remove('active'));
    event.currentTarget.classList.add('active');
}

// --- Auth Handling ---
function showAuth(mode) {
    currentAuthMode = mode;
    document.getElementById('tab-login').classList.toggle('active', mode === 'login');
    document.getElementById('tab-register').classList.toggle('active', mode === 'register');
    document.getElementById('auth-submit').textContent = mode === 'login' ? 'Sign In' : 'Create Account';
    document.getElementById('auth-error').textContent = '';
}

async function handleAuth(event) {
    event.preventDefault();
    const email = document.getElementById('email').value;
    const password = document.getElementById('password').value;
    const errorEl = document.getElementById('auth-error');
    
    const endpoint = currentAuthMode === 'login' ? '/auth/login' : '/auth/register';
    
    try {
        const response = await fetch(`${API_BASE}${endpoint}`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ email, password })
        });
        
        const data = await response.json();
        
        if (!response.ok) throw new Error(data.message || 'Authentication failed');
        
        if (currentAuthMode === 'login') {
            authToken = data.token;
            currentUser = data.user;
            localStorage.setItem('token', authToken);
            document.getElementById('user-email').textContent = currentUser.email;
            showPage('dashboard');
        } else {
            alert('Registration successful! Please login.');
            showAuth('login');
        }
    } catch (err) {
        errorEl.textContent = err.message;
    }
}

async function verifyToken() {
    try {
        const response = await fetch(`${API_BASE}/auth/me`, {
            headers: { 'Authorization': `Bearer ${authToken}` }
        });
        if (response.ok) {
            currentUser = await response.json();
            document.getElementById('user-email').textContent = currentUser.email;
            showPage('dashboard');
        } else {
            logout();
        }
    } catch {
        logout();
    }
}

function logout() {
    authToken = null;
    currentUser = null;
    localStorage.removeItem('token');
    showPage('auth');
}

// --- Dashboard Logic ---
async function loadDashboardData() {
    loadTasks();
    loadExpenses();
}

// --- Task Management ---
async function loadTasks() {
    const list = document.getElementById('task-list');
    try {
        const response = await fetch(`${API_BASE}/tasks`, {
            headers: { 'Authorization': `Bearer ${authToken}` }
        });
        const tasks = await response.json();
        
        list.innerHTML = tasks.length ? '' : '<p class="text-dim">No tasks yet.</p>';
        tasks.forEach(task => {
            const el = document.createElement('div');
            el.className = `task-item glass ${task.completed ? 'completed' : ''}`;
            el.innerHTML = `
                <div class="task-info">
                    <input type="checkbox" ${task.completed ? 'checked' : ''} onchange="toggleTask('${task.id}', this.checked)">
                    <h3>${task.title}</h3>
                </div>
                <button class="delete-btn" onclick="deleteTask('${task.id}')">🗑️</button>
            `;
            list.appendChild(el);
        });
    } catch (err) {
        console.error('Failed to load tasks', err);
    }
}

async function addTask() {
    const input = document.getElementById('new-task-title');
    const title = input.value.trim();
    if (!title) return;
    
    try {
        const response = await fetch(`${API_BASE}/tasks`, {
            method: 'POST',
            headers: { 
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${authToken}`
            },
            body: JSON.stringify({ title })
        });
        if (response.ok) {
            input.value = '';
            loadTasks();
        }
    } catch (err) {
        console.error(err);
    }
}

async function toggleTask(id, completed) {
    try {
        await fetch(`${API_BASE}/tasks/${id}`, {
            method: 'PATCH',
            headers: { 
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${authToken}`
            },
            body: JSON.stringify({ completed })
        });
        loadTasks();
    } catch (err) {
        console.error(err);
    }
}

async function deleteTask(id) {
    if (!confirm('Are you sure?')) return;
    try {
        await fetch(`${API_BASE}/tasks/${id}`, {
            method: 'DELETE',
            headers: { 'Authorization': `Bearer ${authToken}` }
        });
        loadTasks();
    } catch (err) {
        console.error(err);
    }
}

// --- Finance Management ---
async function loadExpenses() {
    const list = document.getElementById('expense-list');
    const totalEl = document.getElementById('total-spending');
    const catEl = document.getElementById('category-summary');
    
    try {
        const [expRes, sumRes] = await Promise.all([
            fetch(`${API_BASE}/expenses`, { headers: { 'Authorization': `Bearer ${authToken}` } }),
            fetch(`${API_BASE}/expenses/summary`, { headers: { 'Authorization': `Bearer ${authToken}` } })
        ]);
        
        const expenses = await expRes.json();
        const summary = await sumRes.json();
        
        totalEl.textContent = `$${summary.total_spending.toFixed(2)}`;
        
        list.innerHTML = expenses.length ? '' : '<p class="text-dim">No expenses yet.</p>';
        expenses.forEach(exp => {
            const el = document.createElement('div');
            el.className = 'expense-item';
            el.innerHTML = `
                <div>
                    <span class="exp-cat">${exp.category}</span>
                    <p class="text-dim" style="font-size: 0.7rem">${new Date(exp.created_at).toLocaleDateString()}</p>
                </div>
                <div class="exp-amt">$${exp.amount.toFixed(2)}</div>
            `;
            list.appendChild(el);
        });

        catEl.innerHTML = '';
        summary.categories.forEach(([cat, amt]) => {
            const pct = (amt / summary.total_spending) * 100;
            const el = document.createElement('div');
            el.className = 'cat-item';
            el.innerHTML = `
                <div class="cat-row">
                    <span>${cat}</span>
                    <span>$${amt.toFixed(2)} (${pct.toFixed(0)}%)</span>
                </div>
                <div class="progress-bar">
                    <div class="progress-fill" style="width: ${pct}%"></div>
                </div>
            `;
            catEl.appendChild(el);
        });
    } catch (err) {
        console.error(err);
    }
}

async function addExpense() {
    const category = document.getElementById('exp-category').value.trim();
    const amount = parseFloat(document.getElementById('exp-amount').value);
    
    if (!category || isNaN(amount)) return;
    
    try {
        const response = await fetch(`${API_BASE}/expenses`, {
            method: 'POST',
            headers: { 
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${authToken}`
            },
            body: JSON.stringify({ category, amount })
        });
        if (response.ok) {
            document.getElementById('exp-category').value = '';
            document.getElementById('exp-amount').value = '';
            loadExpenses();
        }
    } catch (err) {
        console.error(err);
    }
}

// --- AI assistant ---
async function getAISuggestion() {
    showModal('Thinking...');
    try {
        const response = await fetch(`${API_BASE}/ai/suggest`, {
            method: 'POST',
            headers: { 'Authorization': `Bearer ${authToken}` }
        });
        const data = await response.json();
        showModal(data.suggestion, 'AI Smart Suggestion');
    } catch {
        showModal('Failed to get suggestion. Is Groq API key set?', 'AI Error');
    }
}

async function getAIBudgetAnalysis() {
    showModal('Analyzing your spending...');
    try {
        // We reuse the chat endpoint for custom analysis if specific endpoint not available
        const response = await fetch(`${API_BASE}/ai/chat`, {
            method: 'POST',
            headers: { 
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${authToken}`
            },
            body: JSON.stringify({ message: "Analyze my budget based on my recorded expenses and give me brief advice." })
        });
        const data = await response.json();
        showModal(data.response, 'AI Budget Analysis');
    } catch {
        showModal('Failed to get analysis.', 'AI Error');
    }
}

async function sendChat() {
    const input = document.getElementById('chat-input');
    const message = input.value.trim();
    if (!message) return;
    
    const messages = document.getElementById('chat-messages');
    
    // Add user message
    const userEl = document.createElement('div');
    userEl.className = 'user-msg';
    userEl.textContent = message;
    messages.appendChild(userEl);
    input.value = '';
    messages.scrollTop = messages.scrollHeight;

    try {
        const response = await fetch(`${API_BASE}/ai/chat`, {
            method: 'POST',
            headers: { 
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${authToken}` 
            },
            body: JSON.stringify({ message })
        });
        const data = await response.json();
        
        const botEl = document.createElement('div');
        botEl.className = 'bot-msg';
        botEl.textContent = data.response;
        messages.appendChild(botEl);
        messages.scrollTop = messages.scrollHeight;
    } catch (err) {
        console.error(err);
    }
}

// --- UI Helpers ---
function showModal(content, title = 'AI Insight') {
    document.getElementById('modal-title').textContent = title;
    document.getElementById('modal-body').innerText = content;
    document.getElementById('ai-modal').classList.remove('hidden');
}

function closeModal() {
    document.getElementById('ai-modal').classList.add('hidden');
}
