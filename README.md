<div align="center">

# 🎫 InvitationBot
*A modern Discord invite management bot written in Rust*

[![Rust](https://img.shields.io/badge/rust-1.83+-93450a.svg?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![Discord](https://img.shields.io/badge/Discord-bot-5865F2.svg?style=flat-square&logo=discord)](https://discord.com/developers/docs/intro)
[![License](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square)](LICENSE)

</div>

## ✨ Features

- 🔒 **Secure Invite Management**: Generate and track single-use invite links
- 👥 **Role-based Permissions**: Configure invite limits per role
- 📊 **Invite Statistics**: Track who invited whom and view leaderboards
- 🌐 **Web Integration**: Custom invite landing pages
- 🌍 **i18n Support**: Available in English and Traditional Chinese

## 🚀 Quick Start

1. Clone the repository:

```bash
git clone https://github.com/lekoOwO/InvitationBot
cd invitationbot
```

2. Configure your bot:

```bash
mkdir data
cp config.example.yaml data/config.yaml
# Edit config.yaml with your settings
```

Bot must have these permissions in the invite channel:
  - `Attach Files`
  - `Create Instant Invite`
  - `Embed Links`
  - `Manage Channels`
  - `Read Message History`
  - `Send Messages` (Or `Send Messages in Threads`)
  - `Use Slash Commands`

3. Run the bot:

```bash
# Set up the database
echo "DATABASE_URL=sqlite:data/bot.db" > .env
echo "CONFIG_PATH=data/config.yaml" >> .env

cargo run --release
```

## 🛠️ Configuration

The bot is configured through two main files:

### config.yaml
```yaml
bot:
  token: "YOUR_BOT_TOKEN"
  default_invite_max_age: 300  # Default 5 minutes

database:
  path: "data/bot.db"  # SQLite database path

# ... other configurations
```

### Environment Variables
```bash
DATABASE_URL=sqlite:data/bot.db  # SQLite connection string
CONFIG_PATH=data/config.yaml  # Config file path
```

## 🚨 Caution

- The config file is in plain text, so please do not share it with others.
- The config page is accessible by anyone by default, so please set up a proper authentication method.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## 📦 Dependencies

- [poise](https://github.com/serenity-rs/poise) - Discord bot framework
- [sqlx](https://github.com/launchbadge/sqlx) - Async SQL toolkit
- [axum](https://github.com/tokio-rs/axum) - Web framework
- [tokio](https://github.com/tokio-rs/tokio) - Async runtime

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Discord API](https://discord.com/developers/docs/intro)
- [Rust Discord Community](https://discord.gg/rust-lang)

---

<div align="center">

Made with ❤️ and 🦀

</div>