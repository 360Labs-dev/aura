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
    let theme = if desc.contains("dark") {
        "modern.dark"
    } else {
        "modern.light"
    };

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
    if desc.contains("dashboard") || desc.contains("stats") || desc.contains("analytics") {
        return gen_dashboard_app(&app_name, theme);
    }
    if desc.contains("social") || desc.contains("feed") || desc.contains("posts") {
        return gen_social_feed_app(&app_name, theme);
    }
    if desc.contains("music") || desc.contains("player") || desc.contains("audio") {
        return gen_music_player_app(&app_name, theme);
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
            "dashboard" | "analytics" => return "DashboardApp".to_string(),
            "social" | "feed" => return "SocialFeedApp".to_string(),
            "music" | "player" => return "MusicPlayerApp".to_string(),
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
    s.push_str("      todos = todos.append(Todo(title: title))\n\n");
    if has_swipe {
        s.push_str("    action deleteTodo(todo: Todo)\n");
        s.push_str("      todos = todos.remove(todo)\n");
    }

    s
}

fn gen_counter_app(name: &str, theme: &str) -> String {
    format!(
        r#"app {}
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
"#,
        name, theme
    )
}

fn gen_chat_app(name: &str, theme: &str) -> String {
    format!(
        r#"app {}
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
      messages = messages.append(Message(text: input))
      input = ""
"#,
        name, theme
    )
}

fn gen_weather_app(name: &str, theme: &str) -> String {
    format!(
        r#"app {}
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
"#,
        name, theme
    )
}

fn gen_notes_app(name: &str, theme: &str) -> String {
    format!(
        r#"app {}
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
      notes = notes.append(Note(title: "New Note", content: ""))
"#,
        name, theme
    )
}

fn gen_profile_app(name: &str, theme: &str) -> String {
    format!(
        r#"app {}
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
"#,
        name, theme
    )
}

fn gen_timer_app(name: &str, theme: &str) -> String {
    format!(
        r#"app {}
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
"#,
        name, theme
    )
}

fn gen_settings_app(name: &str, theme: &str) -> String {
    format!(
        r#"app {}
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
"#,
        name, theme
    )
}

fn gen_gallery_app(name: &str, theme: &str) -> String {
    format!(
        r#"app {}
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
      photos = photos.append("photo.jpg")
"#,
        name, theme
    )
}

fn gen_login_app(name: &str, theme: &str) -> String {
    format!(
        r#"app {}
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
"#,
        name, theme
    )
}

fn gen_dashboard_app(name: &str, theme: &str) -> String {
    format!(
        r#"app {}
  theme: {}

  model StatCard
    label: text
    value: text
    trend: text

  screen Main
    state date: text = "Today"
    state stats: list[StatCard] = []
    state revenue: int = 45230
    state users: int = 1250
    state orders: int = 342

    view
      column gap.md padding.lg
        row align.center justify.space-between
          heading "Dashboard" size.xl .bold
          button "Export" .surface -> export()
        row gap.md
          button.icon "calendar" title: "Pick Date" -> pickDate()
          text date .secondary
        divider .subtle
        row gap.md
          column flex:1 padding.md .surface .rounded gap.sm
            text "Revenue" .muted size.sm
            text "$45,230" size.xl .bold .accent
            text "↑ 12% this month" size.xs .info
          column flex:1 padding.md .surface .rounded gap.sm
            text "Users" .muted size.sm
            text "1,250" size.xl .bold .warning
            text "↑ 8% this week" size.xs .info
          column flex:1 padding.md .surface .rounded gap.sm
            text "Orders" .muted size.sm
            text "342" size.xl .bold
            text "↑ 23% this month" size.xs .info
        column padding.md .surface .rounded gap.md
          text "Sales Chart" .bold
          box height: 200 .background
            text "Chart placeholder" .muted align.center
        column padding.md .surface .rounded gap.sm
          text "Recent Activity" .bold .medium
          each stats as item
            row padding.xs justify.space-between
              text item.label .secondary
              text item.value

    action export
      return

    action pickDate
      date = "Custom Date"
"#,
        name, theme
    )
}

fn gen_social_feed_app(name: &str, theme: &str) -> String {
    format!(
        r#"app {}
  theme: {}

  model Post
    author: text
    avatar: text
    content: text
    likes: int = 0
    comments: int = 0
    timestamp: timestamp

  screen Main
    state posts: list[Post] = []
    state newPost: text = ""

    view
      column gap.md padding.lg
        heading "Social Feed" size.xl .bold
        row gap.sm
          textfield newPost placeholder: "What's on your mind?" width.fill
          button.icon "paperplane" .accent -> postMessage() -> clearField()
        divider .subtle
        scroll
          column gap.md
            each posts as post
              column padding.md .surface .rounded gap.sm
                row gap.sm align.start
                  avatar post.avatar size.md .circle
                  column flex:1 gap.xs
                    row gap.sm
                      text post.author .bold
                      text post.timestamp size.xs .muted
                    text post.content .medium
                divider .subtle
                row gap.lg padding.top.sm .muted
                  button.slim
                    icon.inline "heart" size.sm
                    text post.likes.toString() size.sm
                    -> likePost(post)
                  button.slim
                    icon.inline "chat.bubble" size.sm
                    text post.comments.toString() size.sm
                    -> replyPost(post)
                  button.slim
                    icon.inline "paperplane" size.sm
                    text "Share" size.sm
                    -> sharePost(post)
        if posts.isEmpty
          column align.center padding.2xl gap.md
            icon "note.text" size.2xl .muted
            text "No posts yet" .muted
            button "Create a post" .accent .pill -> postMessage()

    action postMessage
      posts = posts.append(Post(author: "You", avatar: "me.jpg", content: newPost))

    action clearField
      newPost = ""

    action likePost(post: Post)
      return

    action replyPost(post: Post)
      return

    action sharePost(post: Post)
      return
"#,
        name, theme
    )
}

fn gen_music_player_app(name: &str, theme: &str) -> String {
    format!(
        r#"app {}
  theme: {}

  screen Main
    state playing: bool = false
    state progress: int = 35
    state duration: int = 240
    state track: text = "Midnight Dreams"
    state artist: text = "Luna Eclipse"
    state volume: int = 75

    view
      column gap.xl padding.2xl align.center
        box height: 280 width.fill .background .rounded
          image "album-art.jpg" .fill
            text "Album Art" .muted align.center
        column gap.xs align.center
          text track size.xl .bold
          text artist .secondary
        column width.fill gap.sm
          row gap.sm align.center
            text "0:35" size.xs .muted
            slider progress min: 0 max: 240 step: 1 width.fill
            text "4:00" size.xs .muted
        column gap.md width.fill padding.md gap.md
          row align.center
            text "Volume" size.sm .muted
            slider volume min: 0 max: 100 step: 1 flex:1
            text volume.toString() size.xs .muted
        row gap.md justify.center
          button.icon "shuffle" .surface -> toggleShuffle()
          button.icon "arrow.counterclockwise" .surface -> previous()
          if playing
            button.icon "pause.fill" size.2xl .accent -> pause()
          else
            button.icon "play.fill" size.2xl .accent -> play()
          button.icon "arrow.clockwise" .surface -> next()
          button.icon "repeat" .surface -> toggleRepeat()
        row gap.md padding.top.md
          button "Add to Playlist" .surface .pill width.fill -> addToPlaylist()

    action play
      playing = true

    action pause
      playing = false

    action next
      progress = progress + 30

    action previous
      progress = if progress > 30 then progress - 30 else 0

    action toggleShuffle
      return

    action toggleRepeat
      return

    action addToPlaylist
      return
"#,
        name, theme
    )
}

fn gen_default_app(name: &str, theme: &str, description: &str) -> String {
    format!(
        r#"app {}
  theme: {}

  screen Main
    view
      column gap.lg padding.2xl align.center
        heading "{}" size.xl .bold
        text "{}" .secondary .center
"#,
        name, theme, name, description
    )
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
            "todo app",
            "counter",
            "chat app",
            "weather",
            "notes app",
            "profile page",
            "timer",
            "settings",
            "photo gallery",
            "login screen",
            "dashboard with analytics",
            "social feed",
            "music player",
            "something random",
        ];
        for desc in descriptions {
            let code = sketch(desc);
            let result = crate::parser::parse(&code);
            assert!(
                result.program.is_some(),
                "sketch(\"{}\") produced unparseable code:\n{}\nErrors: {:?}",
                desc,
                code,
                result.errors.iter().map(|e| &e.message).collect::<Vec<_>>()
            );
        }
    }
}
