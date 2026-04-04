//! # Aura Sketch
//!
//! Generates a working `.aura` prototype from a natural language description.
//! Uses keyword matching + templates — no LLM required.
//!
//! Usage: `aura sketch "todo app with dark mode"`

/// Generate a `.aura` source file from a natural language description.
pub fn sketch(description: &str) -> String {
    let desc = description.to_lowercase();
    let app_name = extract_app_name(&desc);
    let theme = if desc.contains("dark") { "modern.dark" } else { "modern.light" };

    // Match against known app patterns
    if desc.contains("todo") || desc.contains("task") || desc.contains("checklist") {
        return gen_todo_app(&app_name, theme, &desc);
    }
    if desc.contains("counter") || desc.contains("clicker") || desc.contains("tally") {
        return gen_counter_app(&app_name, theme);
    }
    if desc.contains("chat") || desc.contains("messenger") || desc.contains("messaging") {
        return gen_chat_app(&app_name, theme);
    }
    if desc.contains("weather") || desc.contains("forecast") || desc.contains("temperature") {
        return gen_weather_app(&app_name, theme);
    }
    if desc.contains("note") || desc.contains("journal") || desc.contains("diary") {
        return gen_notes_app(&app_name, theme);
    }
    if desc.contains("profile") || desc.contains("about me") || desc.contains("portfolio") {
        return gen_profile_app(&app_name, theme);
    }
    if desc.contains("timer") || desc.contains("stopwatch") || desc.contains("countdown") {
        return gen_timer_app(&app_name, theme);
    }
    if desc.contains("settings") || desc.contains("preferences") || desc.contains("config") {
        return gen_settings_app(&app_name, theme);
    }
    if desc.contains("gallery") || desc.contains("photo") || desc.contains("image") {
        return gen_gallery_app(&app_name, theme);
    }
    if desc.contains("login") || desc.contains("auth") || desc.contains("sign in") {
        return gen_login_app(&app_name, theme);
    }

    // Default: a hello world with the description as content
    gen_default_app(&app_name, theme, description)
}

fn extract_app_name(desc: &str) -> String {
    // Try to find a noun that works as an app name
    let words: Vec<&str> = desc.split_whitespace().collect();
    for word in &words {
        match *word {
            "todo" | "task" | "tasks" => return "TodoApp".to_string(),
            "counter" | "clicker" => return "CounterApp".to_string(),
            "chat" | "messenger" => return "ChatApp".to_string(),
            "weather" | "forecast" => return "WeatherApp".to_string(),
            "notes" | "note" | "journal" => return "NotesApp".to_string(),
            "profile" | "portfolio" => return "ProfileApp".to_string(),
            "timer" | "stopwatch" => return "TimerApp".to_string(),
            "settings" => return "SettingsApp".to_string(),
            "gallery" | "photos" => return "GalleryApp".to_string(),
            "login" | "auth" => return "AuthApp".to_string(),
            _ => {}
        }
    }
    "MyApp".to_string()
}

fn gen_todo_app(name: &str, theme: &str, desc: &str) -> String {
    let has_filter = desc.contains("filter") || desc.contains("all") || desc.contains("active");
    let has_swipe = desc.contains("swipe") || desc.contains("delete");
    let has_priority = desc.contains("priority") || desc.contains("important");

    let mut s = format!("app {}\n  theme: {}\n\n", name, theme);

    s.push_str("  model Todo\n    title: text\n    done: bool = false\n");
    if has_priority {
        s.push_str("    priority: enum[low, medium, high] = low\n");
    }
    s.push_str("\n");

    s.push_str("  screen Main\n");
    s.push_str("    state todos: list[Todo] = []\n");
    s.push_str("    state input: text = \"\"\n");
    if has_filter {
        s.push_str("    state filter: enum[all, active, done] = all\n");
    }
    s.push_str("\n    view\n");
    s.push_str("      column gap.md padding.lg\n");
    s.push_str("        heading \"My Tasks\" size.xl .bold\n");
    s.push_str("        row gap.sm\n");
    s.push_str("          textfield input placeholder: \"What needs to be done?\"\n");
    s.push_str("          button \"Add\" .accent -> addTodo(input)\n");
    if has_filter {
        s.push_str("        segmented filter options: [all, active, done]\n");
    }
    s.push_str("        each todos as todo\n");
    s.push_str("          row gap.md align.center padding.sm .surface .rounded\n");
    s.push_str("            checkbox todo.done\n");
    s.push_str("            text todo.title strike: todo.done\n");
    s.push_str("            spacer\n");
    if has_swipe {
        s.push_str("            button.icon \"trash\" .danger -> deleteTodo(todo)\n");
    }
    s.push_str("\n");
    s.push_str("    action addTodo(title: text)\n");
    s.push_str("      todos = todos\n\n");
    if has_swipe {
        s.push_str("    action deleteTodo(todo: Todo)\n");
        s.push_str("      todos = todos\n");
    }

    s
}

fn gen_counter_app(name: &str, theme: &str) -> String {
    format!(r#"app {}
  theme: {}

  screen Main
    state count: int = 0

    view
      column gap.xl padding.2xl align.center
        heading "Counter" size.2xl .bold
        text count size.display .bold .accent
        row gap.md
          button "-" .danger .pill -> decrement()
          button "Reset" .surface .pill -> reset()
          button "+" .accent .pill -> increment()

    action increment
      count = count + 1

    action decrement
      count = count - 1

    action reset
      count = 0
"#, name, theme)
}

fn gen_chat_app(name: &str, theme: &str) -> String {
    format!(r#"app {}
  theme: {}

  model Message
    text: sanitized
    isMine: bool = true
    timestamp: timestamp

  screen Main
    state messages: list[Message] = []
    state input: text = ""

    view
      column
        heading "Chat" size.xl .bold padding.md
        scroll padding.md
          column gap.sm
            each messages as msg
              row justify: if msg.isMine then .end else .start
                text msg.text padding.md .rounded
        row gap.sm padding.md .surface
          textfield input placeholder: "Type a message..."
          button.icon "arrow.up" .accent -> sendMessage()

    action sendMessage
      messages = messages
      input = ""
"#, name, theme)
}

fn gen_weather_app(name: &str, theme: &str) -> String {
    format!(r#"app {}
  theme: {}

  screen Main
    state temperature: int = 72
    state condition: text = "Sunny"
    state city: text = "San Francisco"

    view
      column gap.xl padding.2xl align.center .background
        text city size.lg .secondary
        icon "sun.max" size.3xl .warning
        text temperature size.display .bold
        text condition .secondary .capitalize
        divider .subtle
        heading "Forecast" size.lg .bold padding.top.lg
        row gap.lg justify.center
          column align.center gap.xs
            text "Mon" .muted
            icon "cloud" .secondary
            text "68"
          column align.center gap.xs
            text "Tue" .muted
            icon "cloud.rain" .info
            text "62"
          column align.center gap.xs
            text "Wed" .muted
            icon "sun.max" .warning
            text "75"
"#, name, theme)
}

fn gen_notes_app(name: &str, theme: &str) -> String {
    format!(r#"app {}
  theme: {}

  model Note
    title: text
    content: text
    created: timestamp

  screen Main
    state notes: list[Note] = []

    view
      column gap.md padding.lg
        row align.center
          heading "Notes" size.xl .bold
          spacer
          button.icon "plus" .accent -> addNote()
        each notes as note
          column padding.md gap.xs .surface .rounded
            text note.title .bold
            text note.content .secondary size.sm
            text "Created" size.xs .muted

    action addNote
      notes = notes
"#, name, theme)
}

fn gen_profile_app(name: &str, theme: &str) -> String {
    format!(r#"app {}
  theme: {}

  screen Main
    view
      column gap.lg padding.2xl align.center
        avatar "https://via.placeholder.com/120" size.2xl .circle
        heading "Jane Doe" size.xl .bold
        text "Product Designer" .secondary
        text "San Francisco, CA" .muted size.sm
        divider .subtle
        row gap.2xl padding.top.lg
          column align.center gap.xs
            text "128" size.xl .bold
            text "Posts" size.sm .muted
          column align.center gap.xs
            text "2.4k" size.xl .bold
            text "Followers" size.sm .muted
          column align.center gap.xs
            text "891" size.xl .bold
            text "Following" size.sm .muted
        button "Edit Profile" .accent .pill padding.top.lg -> editProfile()

    action editProfile
      return
"#, name, theme)
}

fn gen_timer_app(name: &str, theme: &str) -> String {
    format!(r#"app {}
  theme: {}

  screen Main
    state seconds: int = 0
    state running: bool = false

    view
      column gap.xl padding.2xl align.center
        heading "Timer" size.xl .bold
        text seconds size.display .bold .mono
        row gap.md
          if running
            button "Pause" .warning .pill -> pause()
          else
            button "Start" .accent .pill -> start()
          button "Reset" .surface .pill -> reset()

    action start
      running = true

    action pause
      running = false

    action reset
      seconds = 0
      running = false
"#, name, theme)
}

fn gen_settings_app(name: &str, theme: &str) -> String {
    format!(r#"app {}
  theme: {}

  screen Main
    state darkMode: bool = false
    state notifications: bool = true
    state volume: int = 75
    state language: text = "English"

    view
      column padding.lg gap.md
        heading "Settings" size.xl .bold
        column .surface .rounded padding.md gap.sm
          toggle darkMode label: "Dark Mode"
          divider .subtle
          toggle notifications label: "Notifications"
          divider .subtle
          row align.center gap.md
            text "Volume" .medium
            slider volume min: 0 max: 100 step: 1
        column .surface .rounded padding.md gap.sm
          row align.center
            text "Language" .medium
            spacer
            text language .secondary
        button "Sign Out" .danger .pill padding.top.lg -> signOut()

    action signOut
      return
"#, name, theme)
}

fn gen_gallery_app(name: &str, theme: &str) -> String {
    format!(r#"app {}
  theme: {}

  screen Main
    state photos: list[text] = []

    view
      column padding.md gap.md
        row align.center
          heading "Gallery" size.xl .bold
          spacer
          button.icon "camera" .accent -> addPhoto()
        grid gap.sm
          each photos as photo
            image photo .rounded
        if photos.isEmpty
          column align.center padding.2xl gap.md
            icon "photo" size.2xl .muted
            text "No photos yet" .muted
            button "Take Photo" .accent .pill -> addPhoto()

    action addPhoto
      photos = photos
"#, name, theme)
}

fn gen_login_app(name: &str, theme: &str) -> String {
    format!(r#"app {}
  theme: {}

  screen Main
    state email: text = ""
    state password: text = ""

    view
      column gap.lg padding.2xl align.center justify.center
        icon "lock.circle" size.3xl .accent
        heading "Welcome Back" size.xl .bold
        text "Sign in to continue" .secondary
        column gap.md width.fill
          textfield email placeholder: "Email address"
          textfield password placeholder: "Password"
          button "Sign In" .accent .pill -> login()
          button.ghost "Forgot Password?" .muted -> forgotPassword()
        row gap.sm padding.top.lg
          text "Don't have an account?" .muted
          button.ghost "Sign Up" .accent -> signUp()

    action login
      return

    action forgotPassword
      return

    action signUp
      return
"#, name, theme)
}

fn gen_default_app(name: &str, theme: &str, description: &str) -> String {
    format!(r#"app {}
  theme: {}

  screen Main
    view
      column gap.lg padding.2xl align.center
        heading "{}" size.xl .bold
        text "{}" .secondary .center
"#, name, theme, name, description)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sketch_todo() {
        let code = sketch("todo app with dark mode and swipe to delete");
        assert!(code.contains("app TodoApp"));
        assert!(code.contains("modern.dark"));
        assert!(code.contains("model Todo"));
        assert!(code.contains("trash"));
    }

    #[test]
    fn test_sketch_counter() {
        let code = sketch("simple counter app");
        assert!(code.contains("app CounterApp"));
        assert!(code.contains("state count: int = 0"));
        assert!(code.contains("action increment"));
    }

    #[test]
    fn test_sketch_chat() {
        let code = sketch("chat messenger app");
        assert!(code.contains("model Message"));
        assert!(code.contains("state messages"));
    }

    #[test]
    fn test_sketch_weather() {
        let code = sketch("weather forecast app");
        assert!(code.contains("temperature"));
        assert!(code.contains("sun.max"));
    }

    #[test]
    fn test_sketch_default() {
        let code = sketch("my awesome project");
        assert!(code.contains("app MyApp"));
    }

    #[test]
    fn test_sketch_generates_parseable_code() {
        // Every sketch template should produce parseable Aura code
        let descriptions = [
            "todo app", "counter", "chat app", "weather", "notes app",
            "profile page", "timer", "settings", "photo gallery", "login screen",
            "something random",
        ];
        for desc in descriptions {
            let code = sketch(desc);
            let result = crate::parser::parse(&code);
            assert!(
                result.program.is_some(),
                "sketch(\"{}\") produced unparseable code:\n{}\nErrors: {:?}",
                desc, code,
                result.errors.iter().map(|e| &e.message).collect::<Vec<_>>()
            );
        }
    }
}
