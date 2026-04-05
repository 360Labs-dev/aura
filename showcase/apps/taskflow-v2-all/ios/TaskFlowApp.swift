import SwiftUI

struct Task: Identifiable, Hashable {
    let id = UUID()
    var title: String
    var done: Bool = false
    var category: String = "general"
}

struct MainView: View {
    @State private var tasks: [Task] = []
    @State private var input: String = ""
    @State private var filter: String = "all"

    var body: some View {
        VStack(spacing: 16) {
            VStack(spacing: 2) {
                Text("TaskFlow")
                    .font(.title)
                    .fontWeight(.bold)
                    .font(.system(size: 24))
                    .fontWeight(.bold)
                Text("Get things done.")
            }
                .padding(.top, 16)
                .padding(.bottom, 16)
            VStack(spacing: 4) {
                HStack(spacing: 4) {
                    TextField("Add a new task...", text: $input)
                    Button("Add", action: { addTask(input) })
                        .buttonStyle(.borderedProminent)
                }
            }
                .padding(8)
            HStack(spacing: 4) {
                Button("All", action: { showAll() })
                    .buttonStyle(.borderedProminent)
                    .clipShape(Capsule())
                Button("Active", action: { showActive() })
                    .buttonStyle(.borderedProminent)
                    .clipShape(Capsule())
                Button("Done", action: { showDone() })
                    .buttonStyle(.borderedProminent)
                    .clipShape(Capsule())
            }
            Text("Showing tasks")
            VStack(spacing: 2) {
                ForEach(tasks, id: \.self) { task in
                    HStack(spacing: 8) {
                        Toggle(isOn: $task.done) {
                            EmptyView()
                        }
                        .toggleStyle(.checkbox)
                        Text(task.title)
                        Spacer()
                        Text(task.category)
                        Button(action: { deleteTask(task) }) {
                            Image(systemName: "trash")
                        }
                            .tint(.red)
                    }
                        .padding(8)
                }
            }
            if tasks.isEmpty {
                VStack(spacing: 4) {
                    Text("No tasks yet")
                        .foregroundColor(.gray)
                    Text("Type something above and hit Add")
                }
                    .padding(32)
            }
            HStack(spacing: 8) {
                Text("Manage your tasks")
                Button("Clear completed", action: { clearDone() })
            }
                .padding(.top, 8)
        }
            .padding(16)
    }

    func addTask(title: String) {
        tasks = tasks.append(Task(title: title))
        input = ""
    }

    func deleteTask(task: Task) {
        tasks = tasks.remove(task)
    }

    func clearDone() {
        tasks = tasks.filter({ t in !t.done })
    }

    func showAll() {
        filter = "all"
    }

    func showActive() {
        filter = "active"
    }

    func showDone() {
        filter = "done"
    }
}

@main
struct TaskFlowApp: App {
    var body: some Scene {
        WindowGroup {
            NavigationStack {
                MainView()
            }
        }
    }
}
