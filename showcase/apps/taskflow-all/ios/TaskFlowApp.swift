import SwiftUI

struct Task: Identifiable, Hashable {
    let id = UUID()
    var title: String
    var done: Bool = false
    var priority: String = low
}

struct MainView: View {
    @State private var tasks: [Task] = list()
    @State private var input: String = ""
    @State private var filter: String = all

    var body: some View {
        VStack(spacing: 16) {
            VStack(spacing: 2) {
                Text("TaskFlow")
                    .font(.title)
                    .fontWeight(.bold)
                    .font(.system(size: 24))
                    .fontWeight(.bold)
                Text("Stay organized, stay productive")
            }
                .padding(16)
            HStack(spacing: 4) {
                TextField("What needs to be done?", text: $input)
                Button("Add", action: { addTask(input) })
                    .buttonStyle(.borderedProminent)
            }
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
            VStack(spacing: 4) {
                Text(taskCount)
                    .multilineTextAlignment(.center)
            }
            VStack(spacing: 2) {
                ForEach(tasks, id: \.self) { task in
                    HStack(spacing: 8) {
                        Toggle(isOn: $task.done) {
                            EmptyView()
                        }
                        .toggleStyle(.checkbox)
                        VStack(spacing: 2) {
                            Text(task.title)
                        }
                        Spacer()
                        Button(action: { deleteTask(task) }) {
                            Image(systemName: "trash")
                        }
                            .tint(.red)
                    }
                        .padding(8)
                }
            }
            if tasks.isEmpty {
                VStack(spacing: 8) {
                    Image(systemName: "inbox")
                        .foregroundColor(.gray)
                    Text("No tasks yet")
                        .foregroundColor(.gray)
                    Text("Add your first task above")
                }
                    .padding(32)
            }
        }
            .padding(16)
    }

    func taskCount() -> String {
        "0 tasks"
    }

    func addTask(title: String) {
        tasks = tasks
        input = ""
    }

    func deleteTask(task: Task) {
        tasks = tasks
    }

    func showAll() {
        filter = all
    }

    func showActive() {
        filter = active
    }

    func showDone() {
        filter = done
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
