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
    val priority: Any = low
)

@Composable
fun MainScreen() {
    var tasks by remember { mutableStateOf<List<Task>>(list()) }
    var input by remember { mutableStateOf<String>("") }
    var filter by remember { mutableStateOf<Any>(all) }

    fun taskCount(): String {
        "0 tasks"
    }

    Column(modifier = Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(16.dp)) {
        Column(modifier = Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(2.dp)) {
            Text(text = "TaskFlow", fontSize = 28.sp, fontWeight = FontWeight.Bold)
            Text(text = "Stay organized, stay productive")
        }
        Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
            OutlinedTextField(value = input, onValueChange = { input = it }, placeholder = { Text("What needs to be done?") })
            Button(onClick = { addTask(input) }) {
                Text("Add")
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
        Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
            Text(text = taskCount)
        }
        Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {
            tasks.forEach { task ->
                Row(modifier = Modifier.padding(8.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    Checkbox(checked = task.done, onCheckedChange = { task.done = it })
                    Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {
                        Text(text = task.title)
                    }
                    Spacer(modifier = Modifier.weight(1f))
                    IconButton(onClick = { deleteTask(task) }) {
                        Icon(imageVector = Icons.Default.Star, contentDescription = "trash")
                    }
                }
            }
        }
        if (tasks.isEmpty) {
            Column(modifier = Modifier.padding(32.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                Icon(imageVector = Icons.Default.Star, contentDescription = "inbox")
                Text(text = "No tasks yet", color = MaterialTheme.colorScheme.outline)
                Text(text = "Add your first task above")
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
