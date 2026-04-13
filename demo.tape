# VHS recording script for solscope demo
# Usage: HELIUS_API_KEY=your_key vhs demo.tape

Output assets/demo.gif
Set Theme "Catppuccin Mocha"
Set FontSize 14
Set Width 1200
Set Height 700
Set Padding 10
Set TypingSpeed 40ms

# Launch solscope
Type "./target/release/solscope --wallet CKs1E69a2e9TmH4mKKLrXFF8kD3ZnwKjoEuXa6sz9WqX"
Enter
Sleep 4s

# Portfolio tab - scroll through holdings
Sleep 1s
Type "j"
Sleep 400ms
Type "j"
Sleep 400ms
Type "j"
Sleep 400ms

# Cycle sort modes
Type "s"
Sleep 1.5s
Type "s"
Sleep 1.5s
Type "s"
Sleep 1s

# Switch to Transactions tab
Type "2"
Sleep 3s

# Scroll through transactions
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Type "j"
Sleep 1.5s

# Switch to Whale Tracker
Type "3"
Sleep 1.5s

# Add a whale wallet
Type "a"
Sleep 500ms
Type "CKs1E69a2e9TmH4mKKLrXFF8kD3ZnwKjoEuXa6sz9WqX"
Enter
Sleep 500ms
Type "whale1"
Enter
Sleep 3s

# Switch to Token Lookup
Type "4"
Sleep 1s

# Search for Jupiter token
Type "/"
Sleep 300ms
Type "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN"
Enter
Sleep 4s

# Copy mint address
Type "y"
Sleep 2s

# Show help
Type "?"
Sleep 3s

# Back to portfolio
Type "1"
Sleep 2s

# Quit
Type "q"
Sleep 500ms
