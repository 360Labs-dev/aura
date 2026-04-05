package com.aura.taskflow

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.*
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

data class Task(
    val title: String,
    val done: Boolean = false,
    val category: String = "general"
)

@Composable
fun MainScreen() {
    var tasks by remember { mutableStateOf<List<Task>>(listOf()) }
    var input by remember { mutableStateOf<String>("") }
    var filter by remember { mutableStateOf<String>("all") }

    Column(modifier = Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(16.dp)) {
        Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {
            Text(text = "TaskFlow", fontSize = 28.sp, fontWeight = FontWeight.Bold)
            Text(text = "Get things done.")
        }
        Column(modifier = Modifier.padding(8.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
            Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                OutlinedTextField(value = input, onValueChange = { input = it }, placeholder = { Text("Add a new task...") })
                Button(onClick = { addTask(input) }) {
                    Text("Add")
                }
            }
        }
        Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
            Button(onClick = { showAll() }) {
                Text("All")
            }
            Button(onClick = { showActive() }) {
                Text("Active")
            }
            Button(onClick = { showDone() }) {
                Text("Done")
            }
        }
        Text(text = "Showing tasks")
        Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {
            tasks.forEach { task ->
                Row(modifier = Modifier.padding(8.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    Checkbox(checked = task.done, onCheckedChange = { task.done = it })
                    Text(text = task.title)
                    Spacer(modifier = Modifier.weight(1f))
                    Text(text = task.category)
                    IconButton(onClick = { deleteTask(task) }) {
                        Icon(imageVector = Icons.Default.Star, contentDescription = "trash")
                    }
                }
            }
        }
        if (tasks.isEmpty) {
            Column(modifier = Modifier.padding(32.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                Text(text = "No tasks yet", color = MaterialTheme.colorScheme.outline)
                Text(text = "Type something above and hit Add")
            }
        }
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Text(text = "Manage your tasks")
            Button(onClick = { clearDone() }) {
                Text("Clear completed")
            }
        }
    }
}

class TaskFlowActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            MaterialTheme {
                MainScreen()
            }
        }
    }
}
