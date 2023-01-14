# Example

### config.toml создаётся автоматически при первом запуске
***
Описание config.toml

```toml
# Порт сервера (int)
"port" = 3000
# Спам заглушка, пароль (string) 
"auth" = ""

# ID telegram-чата (int) 
"id" = 
# Токен telegram-бота (sring)
"token" = ""
```

[ID чата можно получить тут](https://t.me/username_to_id_bot)

[Токен бота можно получить тут](https://t.me/BotFather)




```bash
curl  -H 'Content-Type: application/json' -d '{"auth":"","message":""}' http://<host>:<port>/message
```

### dev.http
```
POST http://<host>:<port>/message
Content-Type: application/json

{
    "auth": "",
    "message": "your_message"
}
```