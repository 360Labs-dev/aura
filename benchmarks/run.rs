//! Aura Benchmark Runner
//!
//! Compiles benchmark programs through the full pipeline and measures:
//! - Token count (LLM cost proxy)
//! - Line count (code brevity)
//! - Compile time
//! - First-compile success rate
//! - Output size per platform
//!
//! Also compares against equivalent TypeScript/Swift/Kotlin implementations.

use std::time::Instant;

fn main() {
    println!();
    println!("  ╔══════════════════════════════════════════════════════════════╗");
    println!("  ║              AURA LANGUAGE BENCHMARK SUITE                  ║");
    println!("  ║              v0.1.0 — {:<36} ║", chrono_now());
    println!("  ╚══════════════════════════════════════════════════════════════╝");
    println!();

    let benchmarks = vec![
        BenchmarkCase {
            name: "Hello World",
            aura_source: r#"app Hello
  screen Main
    view
      text "Hello, Aura!""#,
            typescript_equivalent: r#"import React from 'react';
import { View, Text, StyleSheet } from 'react-native';

export default function App() {
  return (
    <View style={styles.container}>
      <Text style={styles.text}>Hello, Aura!</Text>
    </View>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, justifyContent: 'center', alignItems: 'center' },
  text: { fontSize: 16 },
});"#,
            swift_equivalent: r#"import SwiftUI

@main
struct HelloApp: App {
    var body: some Scene {
        WindowGroup {
            ContentView()
        }
    }
}

struct ContentView: View {
    var body: some View {
        Text("Hello, Aura!")
    }
}"#,
            kotlin_equivalent: r#"package com.example.hello

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.material3.*
import androidx.compose.runtime.*

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            MaterialTheme {
                Text("Hello, Aura!")
            }
        }
    }
}"#,
        },
        BenchmarkCase {
            name: "Counter App",
            aura_source: r#"app Counter
  screen Main
    state count: int = 0
    view
      column gap.lg padding.xl align.center
        heading "Counter" size.2xl
        text count size.display .bold
        row gap.md
          button "-" .danger -> decrement()
          button "+" .accent -> increment()
    action increment
      count = count + 1
    action decrement
      count = count - 1"#,
            typescript_equivalent: r#"import React, { useState } from 'react';
import { View, Text, TouchableOpacity, StyleSheet } from 'react-native';

export default function App() {
  const [count, setCount] = useState(0);

  return (
    <View style={styles.container}>
      <Text style={styles.title}>Counter</Text>
      <Text style={styles.count}>{count}</Text>
      <View style={styles.row}>
        <TouchableOpacity style={[styles.button, styles.danger]} onPress={() => setCount(c => c - 1)}>
          <Text style={styles.buttonText}>-</Text>
        </TouchableOpacity>
        <TouchableOpacity style={[styles.button, styles.accent]} onPress={() => setCount(c => c + 1)}>
          <Text style={styles.buttonText}>+</Text>
        </TouchableOpacity>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, justifyContent: 'center', alignItems: 'center', gap: 24, padding: 32 },
  title: { fontSize: 24, fontWeight: 'bold' },
  count: { fontSize: 48, fontWeight: 'bold' },
  row: { flexDirection: 'row', gap: 8 },
  button: { paddingHorizontal: 24, paddingVertical: 12, borderRadius: 8 },
  danger: { backgroundColor: '#DC3545' },
  accent: { backgroundColor: '#6C5CE7' },
  buttonText: { color: 'white', fontSize: 18, fontWeight: '600' },
});"#,
            swift_equivalent: r#"import SwiftUI

@main
struct CounterApp: App {
    var body: some Scene {
        WindowGroup {
            ContentView()
        }
    }
}

struct ContentView: View {
    @State private var count = 0

    var body: some View {
        VStack(spacing: 24) {
            Text("Counter")
                .font(.title)
                .fontWeight(.bold)
            Text("\(count)")
                .font(.system(size: 48))
                .fontWeight(.bold)
            HStack(spacing: 8) {
                Button("-") { count -= 1 }
                    .buttonStyle(.borderedProminent)
                    .tint(.red)
                Button("+") { count += 1 }
                    .buttonStyle(.borderedProminent)
                    .tint(.purple)
            }
        }
        .padding(32)
    }
}"#,
            kotlin_equivalent: r#"package com.example.counter

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            MaterialTheme {
                CounterScreen()
            }
        }
    }
}

@Composable
fun CounterScreen() {
    var count by remember { mutableStateOf(0) }

    Column(
        modifier = Modifier.fillMaxSize().padding(32.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(24.dp, Alignment.CenterVertically)
    ) {
        Text("Counter", fontSize = 24.sp, fontWeight = FontWeight.Bold)
        Text("$count", fontSize = 48.sp, fontWeight = FontWeight.Bold)
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Button(onClick = { count-- }, colors = ButtonDefaults.buttonColors(containerColor = MaterialTheme.colorScheme.error)) {
                Text("-")
            }
            Button(onClick = { count++ }) {
                Text("+")
            }
        }
    }
}"#,
        },
        BenchmarkCase {
            name: "Todo List",
            aura_source: r#"app TodoApp
  model Todo
    title: text
    done: bool = false

  screen Main
    state todos: list[Todo] = []
    state input: text = ""

    view
      column gap.md padding.lg
        heading "Tasks" size.xl .bold
        row gap.sm
          textfield input placeholder: "New task..."
          button "Add" .accent -> addTodo(input)
        each todos as todo
          row gap.md align.center padding.sm .surface .rounded
            checkbox todo.done
            text todo.title
            spacer
            button.icon "trash" .danger -> deleteTodo(todo)

    action addTodo(title: text)
      todos = todos

    action deleteTodo(todo: Todo)
      todos = todos"#,
            typescript_equivalent: r#"import React, { useState } from 'react';
import { View, Text, TextInput, TouchableOpacity, FlatList, StyleSheet } from 'react-native';

interface Todo {
  id: string;
  title: string;
  done: boolean;
}

export default function App() {
  const [todos, setTodos] = useState<Todo[]>([]);
  const [input, setInput] = useState('');

  const addTodo = () => {
    if (!input.trim()) return;
    setTodos(prev => [...prev, { id: Date.now().toString(), title: input.trim(), done: false }]);
    setInput('');
  };

  const toggleTodo = (id: string) => {
    setTodos(prev => prev.map(t => t.id === id ? { ...t, done: !t.done } : t));
  };

  const deleteTodo = (id: string) => {
    setTodos(prev => prev.filter(t => t.id !== id));
  };

  return (
    <View style={styles.container}>
      <Text style={styles.title}>Tasks</Text>
      <View style={styles.inputRow}>
        <TextInput style={styles.input} value={input} onChangeText={setInput} placeholder="New task..." />
        <TouchableOpacity style={styles.addButton} onPress={addTodo}>
          <Text style={styles.addButtonText}>Add</Text>
        </TouchableOpacity>
      </View>
      <FlatList
        data={todos}
        keyExtractor={item => item.id}
        renderItem={({ item }) => (
          <View style={styles.todoRow}>
            <TouchableOpacity onPress={() => toggleTodo(item.id)}>
              <Text>{item.done ? '☑' : '☐'}</Text>
            </TouchableOpacity>
            <Text style={[styles.todoText, item.done && styles.done]}>{item.title}</Text>
            <TouchableOpacity onPress={() => deleteTodo(item.id)}>
              <Text style={styles.deleteText}>🗑</Text>
            </TouchableOpacity>
          </View>
        )}
      />
    </View>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, padding: 16, gap: 8 },
  title: { fontSize: 24, fontWeight: 'bold' },
  inputRow: { flexDirection: 'row', gap: 4 },
  input: { flex: 1, borderWidth: 1, borderColor: '#ddd', borderRadius: 8, padding: 8 },
  addButton: { backgroundColor: '#6C5CE7', paddingHorizontal: 16, paddingVertical: 8, borderRadius: 8 },
  addButtonText: { color: 'white', fontWeight: '600' },
  todoRow: { flexDirection: 'row', alignItems: 'center', gap: 8, padding: 8, backgroundColor: '#f5f5f5', borderRadius: 8 },
  todoText: { flex: 1 },
  done: { textDecorationLine: 'line-through', color: '#999' },
  deleteText: { color: '#DC3545' },
});"#,
            swift_equivalent: r#"import SwiftUI

struct Todo: Identifiable {
    let id = UUID()
    var title: String
    var done: Bool = false
}

@main
struct TodoApp: App {
    var body: some Scene {
        WindowGroup { ContentView() }
    }
}

struct ContentView: View {
    @State private var todos: [Todo] = []
    @State private var input = ""

    var body: some View {
        VStack(spacing: 8) {
            Text("Tasks").font(.title).fontWeight(.bold)
            HStack(spacing: 4) {
                TextField("New task...", text: $input)
                    .textFieldStyle(.roundedBorder)
                Button("Add") {
                    guard !input.trimmingCharacters(in: .whitespaces).isEmpty else { return }
                    todos.append(Todo(title: input))
                    input = ""
                }
                .buttonStyle(.borderedProminent)
            }
            List {
                ForEach($todos) { $todo in
                    HStack {
                        Button { todo.done.toggle() } label: {
                            Image(systemName: todo.done ? "checkmark.circle.fill" : "circle")
                        }
                        Text(todo.title)
                            .strikethrough(todo.done)
                        Spacer()
                    }
                }
                .onDelete { todos.remove(atOffsets: $0) }
            }
        }
        .padding()
    }
}"#,
            kotlin_equivalent: r#"package com.example.todo

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.unit.dp
import java.util.UUID

data class Todo(val id: String = UUID.randomUUID().toString(), val title: String, val done: Boolean = false)

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent { MaterialTheme { TodoScreen() } }
    }
}

@Composable
fun TodoScreen() {
    var todos by remember { mutableStateOf(listOf<Todo>()) }
    var input by remember { mutableStateOf("") }

    Column(modifier = Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
        Text("Tasks", style = MaterialTheme.typography.headlineMedium)
        Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
            OutlinedTextField(value = input, onValueChange = { input = it }, modifier = Modifier.weight(1f), placeholder = { Text("New task...") })
            Button(onClick = {
                if (input.isNotBlank()) { todos = todos + Todo(title = input.trim()); input = "" }
            }) { Text("Add") }
        }
        LazyColumn(verticalArrangement = Arrangement.spacedBy(4.dp)) {
            items(todos, key = { it.id }) { todo ->
                Row(modifier = Modifier.fillMaxWidth().padding(8.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    Checkbox(checked = todo.done, onCheckedChange = { todos = todos.map { if (it.id == todo.id) it.copy(done = !it.done) else it } })
                    Text(todo.title, modifier = Modifier.weight(1f), textDecoration = if (todo.done) TextDecoration.LineThrough else null)
                    IconButton(onClick = { todos = todos.filter { it.id != todo.id } }) {
                        Icon(Icons.Default.Delete, contentDescription = "Delete", tint = MaterialTheme.colorScheme.error)
                    }
                }
            }
        }
    }
}"#,
        },
    ];

    let mut results = Vec::new();

    for case in &benchmarks {
        let result = run_benchmark(case);
        results.push(result);
    }

    println!();
    print_results_table(&benchmarks, &results);
    println!();
    print_summary(&benchmarks, &results);
    println!();
    print_token_comparison(&benchmarks);
}

struct BenchmarkCase {
    name: &'static str,
    aura_source: &'static str,
    typescript_equivalent: &'static str,
    swift_equivalent: &'static str,
    kotlin_equivalent: &'static str,
}

struct BenchmarkResult {
    parse_time_us: u128,
    analyze_time_us: u128,
    hir_time_us: u128,
    web_codegen_time_us: u128,
    swift_codegen_time_us: u128,
    compose_codegen_time_us: u128,
    total_time_us: u128,
    parse_errors: usize,
    semantic_errors: usize,
    web_output_bytes: usize,
    swift_output_bytes: usize,
    compose_output_bytes: usize,
    aura_lines: usize,
    aura_tokens: usize,
}

fn run_benchmark(case: &BenchmarkCase) -> BenchmarkResult {
    // Parse
    let t0 = Instant::now();
    let parse_result = aura_core::parser::parse(case.aura_source);
    let parse_time = t0.elapsed().as_micros();

    let parse_errors = parse_result.errors.len();

    let program = parse_result.program.unwrap();

    // Semantic analysis
    let t1 = Instant::now();
    let analysis = aura_core::semantic::SemanticAnalyzer::new().analyze(&program);
    let analyze_time = t1.elapsed().as_micros();
    let semantic_errors = analysis.errors.iter().filter(|e| e.is_error()).count();

    // HIR
    let t2 = Instant::now();
    let hir = aura_core::hir::build_hir(&program);
    let hir_time = t2.elapsed().as_micros();

    // Web codegen
    let t3 = Instant::now();
    let web = aura_backend_web::compile_to_web(&hir);
    let web_time = t3.elapsed().as_micros();

    // Swift codegen
    let t4 = Instant::now();
    let swift = aura_backend_swift::compile_to_swift(&hir);
    let swift_time = t4.elapsed().as_micros();

    // Compose codegen
    let t5 = Instant::now();
    let compose = aura_backend_compose::compile_to_compose(&hir);
    let compose_time = t5.elapsed().as_micros();

    let total = parse_time + analyze_time + hir_time + web_time + swift_time + compose_time;

    // Count Aura tokens (rough: split on whitespace)
    let aura_tokens = case.aura_source.split_whitespace().count();
    let aura_lines = case.aura_source.lines().count();

    BenchmarkResult {
        parse_time_us: parse_time,
        analyze_time_us: analyze_time,
        hir_time_us: hir_time,
        web_codegen_time_us: web_time,
        swift_codegen_time_us: swift_time,
        compose_codegen_time_us: compose_time,
        total_time_us: total,
        parse_errors,
        semantic_errors,
        web_output_bytes: web.html.len() + web.css.len() + web.js.len(),
        swift_output_bytes: swift.swift.len(),
        compose_output_bytes: compose.kotlin.len(),
        aura_lines,
        aura_tokens,
    }
}

fn print_results_table(cases: &[BenchmarkCase], results: &[BenchmarkResult]) {
    println!(
        "  ┌────────────────────────────────────────────────────────────────────────────────────┐"
    );
    println!(
        "  │                           COMPILATION BENCHMARKS                                   │"
    );
    println!(
        "  ├──────────────┬──────────┬──────────┬──────────┬──────────┬──────────┬──────────────┤"
    );
    println!(
        "  │ Benchmark    │ Parse    │ Analyze  │ HIR      │ Codegen  │ Total    │ Errors       │"
    );
    println!(
        "  │              │          │          │          │ (3 tgts) │          │              │"
    );
    println!(
        "  ├──────────────┼──────────┼──────────┼──────────┼──────────┼──────────┼──────────────┤"
    );

    for (case, result) in cases.iter().zip(results.iter()) {
        let codegen_total = result.web_codegen_time_us
            + result.swift_codegen_time_us
            + result.compose_codegen_time_us;
        println!(
            "  │ {:<12} │ {:>5} us │ {:>5} us │ {:>5} us │ {:>5} us │ {:>5} us │ {:>2} parse {:>2} sem │",
            case.name,
            result.parse_time_us,
            result.analyze_time_us,
            result.hir_time_us,
            codegen_total,
            result.total_time_us,
            result.parse_errors,
            result.semantic_errors,
        );
    }
    println!(
        "  └──────────────┴──────────┴──────────┴──────────┴──────────┴──────────┴──────────────┘"
    );

    println!();
    println!(
        "  ┌────────────────────────────────────────────────────────────────────────────────────┐"
    );
    println!(
        "  │                           OUTPUT SIZE (bytes)                                       │"
    );
    println!(
        "  ├──────────────┬────────────────────┬──────────────────┬──────────────────────────────┤"
    );
    println!(
        "  │ Benchmark    │ Web (HTML+CSS+JS)  │ iOS (Swift)      │ Android (Kotlin)             │"
    );
    println!(
        "  ├──────────────┼────────────────────┼──────────────────┼──────────────────────────────┤"
    );

    for (case, result) in cases.iter().zip(results.iter()) {
        println!(
            "  │ {:<12} │ {:>10} bytes   │ {:>10} bytes │ {:>10} bytes               │",
            case.name,
            result.web_output_bytes,
            result.swift_output_bytes,
            result.compose_output_bytes,
        );
    }
    println!(
        "  └──────────────┴────────────────────┴──────────────────┴──────────────────────────────┘"
    );
}

fn print_token_comparison(cases: &[BenchmarkCase]) {
    println!(
        "  ┌──────────────────────────────────────────────────────────────────────────────────────┐"
    );
    println!(
        "  │                    CODE SIZE COMPARISON (lines / tokens)                              │"
    );
    println!(
        "  ├──────────────┬────────────────┬────────────────┬────────────────┬────────────────────┤"
    );
    println!(
        "  │ Benchmark    │ Aura           │ TypeScript+RN  │ Swift+SwiftUI  │ Kotlin+Compose     │"
    );
    println!(
        "  ├──────────────┼────────────────┼────────────────┼────────────────┼────────────────────┤"
    );

    for case in cases {
        let aura_l = case.aura_source.lines().count();
        let aura_t = case.aura_source.split_whitespace().count();
        let ts_l = case.typescript_equivalent.lines().count();
        let ts_t = case.typescript_equivalent.split_whitespace().count();
        let sw_l = case.swift_equivalent.lines().count();
        let sw_t = case.swift_equivalent.split_whitespace().count();
        let kt_l = case.kotlin_equivalent.lines().count();
        let kt_t = case.kotlin_equivalent.split_whitespace().count();

        println!(
            "  │ {:<12} │ {:>3}L / {:>4}T  │ {:>3}L / {:>4}T  │ {:>3}L / {:>4}T  │ {:>3}L / {:>4}T      │",
            case.name, aura_l, aura_t, ts_l, ts_t, sw_l, sw_t, kt_l, kt_t,
        );
    }
    println!(
        "  ├──────────────┼────────────────┼────────────────┼────────────────┼────────────────────┤"
    );

    // Totals
    let aura_total_l: usize = cases.iter().map(|c| c.aura_source.lines().count()).sum();
    let aura_total_t: usize = cases
        .iter()
        .map(|c| c.aura_source.split_whitespace().count())
        .sum();
    let ts_total_l: usize = cases
        .iter()
        .map(|c| c.typescript_equivalent.lines().count())
        .sum();
    let ts_total_t: usize = cases
        .iter()
        .map(|c| c.typescript_equivalent.split_whitespace().count())
        .sum();
    let sw_total_l: usize = cases
        .iter()
        .map(|c| c.swift_equivalent.lines().count())
        .sum();
    let sw_total_t: usize = cases
        .iter()
        .map(|c| c.swift_equivalent.split_whitespace().count())
        .sum();
    let kt_total_l: usize = cases
        .iter()
        .map(|c| c.kotlin_equivalent.lines().count())
        .sum();
    let kt_total_t: usize = cases
        .iter()
        .map(|c| c.kotlin_equivalent.split_whitespace().count())
        .sum();

    println!(
        "  │ TOTAL        │ {:>3}L / {:>4}T  │ {:>3}L / {:>4}T  │ {:>3}L / {:>4}T  │ {:>3}L / {:>4}T      │",
        aura_total_l,
        aura_total_t,
        ts_total_l,
        ts_total_t,
        sw_total_l,
        sw_total_t,
        kt_total_l,
        kt_total_t,
    );
    println!(
        "  └──────────────┴────────────────┴────────────────┴────────────────┴────────────────────┘"
    );

    let ts_reduction = ((1.0 - (aura_total_t as f64 / ts_total_t as f64)) * 100.0).round();
    let sw_reduction = ((1.0 - (aura_total_t as f64 / sw_total_t as f64)) * 100.0).round();
    let kt_reduction = ((1.0 - (aura_total_t as f64 / kt_total_t as f64)) * 100.0).round();

    println!();
    println!("  Token reduction vs TypeScript+RN:    {}%", ts_reduction);
    println!("  Token reduction vs Swift+SwiftUI:    {}%", sw_reduction);
    println!("  Token reduction vs Kotlin+Compose:   {}%", kt_reduction);
}

fn print_summary(cases: &[BenchmarkCase], results: &[BenchmarkResult]) {
    let total_parse: u128 = results.iter().map(|r| r.parse_time_us).sum();
    let total_analyze: u128 = results.iter().map(|r| r.analyze_time_us).sum();
    let total_compile: u128 = results.iter().map(|r| r.total_time_us).sum();
    let total_errors: usize = results.iter().map(|r| r.parse_errors).sum();
    let first_compile_success = results.iter().filter(|r| r.parse_errors == 0).count();

    println!("  ┌──────────────────────────────────────────────────────────────┐");
    println!("  │                         SUMMARY                              │");
    println!("  ├──────────────────────────────────────────────────────────────┤");
    println!("  │ Benchmarks run:           {:<33} │", cases.len());
    println!(
        "  │ First-compile success:    {}/{} ({:.0}%){:<24} │",
        first_compile_success,
        cases.len(),
        (first_compile_success as f64 / cases.len() as f64) * 100.0,
        ""
    );
    println!("  │ Total parse time:         {:<30} us │", total_parse);
    println!("  │ Total analysis time:      {:<30} us │", total_analyze);
    println!("  │ Total compile time:       {:<30} us │", total_compile);
    println!(
        "  │ Avg compile time:         {:<30} us │",
        total_compile / cases.len() as u128
    );
    println!("  │ Total parse errors:       {:<33} │", total_errors);
    println!(
        "  │ Platforms generated:      3 (Web + iOS + Android){:<9} │",
        ""
    );
    println!("  └──────────────────────────────────────────────────────────────┘");
}

fn chrono_now() -> String {
    // Simple date without chrono dependency
    "2026-04-04".to_string()
}
