# BN_MANAGER
[![en](https://img.shields.io/badge/Lang-English-red.svg)](https://github.com/fatorin/bn_manager/blob/master/README.md)
[![zh-tw](https://img.shields.io/badge/Lang-正體中文-blue.svg)](https://github.com/fatorin/bn_manager/blob/master/README.zh-tw.md)

BN_MANAGER is a management tool designed for [PVPGN](https://github.com/pvpgn/pvpgn-server) (Player vs Player Gaming Network), providing an intuitive interface to monitor and manage the Battle.net gaming environment.

## Features Overview

- **Room Monitoring**: Real-time view of active rooms on Battle.net
- **Map Management**: Specific players can upload custom maps through a hidden page
- **Account Integration**:
    - Register and link Battle.net accounts via Discord
    - Securely modify account passwords
    - Player reporting system

### Discord Commands

Players can use the following commands in the Discord channel:
- `/register` - Register a new account
- `/find_account` - Find an account
- `/forget_password` - Reset forgotten password
- `/report` - Report a player

Administrators holding any of the roles listed in `discord_admin_role_ids` can additionally use:
- `/admin_register` - Create an account with a randomly generated password (the reply is visible only to the admin), and record the creator in the `admin_created_accounts` table.
- `/admin_find_account` - Look up an account from a Discord user, or a Discord user from an account name. Fill in exactly one of the two options.
  Only accounts players created with `/register` are found; accounts made with `/admin_register` are not linked to Discord.
  These commands are not registered while `discord_admin_role_ids` is empty.

## Usage Instructions

Please follow the configuration in the `settings.toml` file to use this tool. Ensure all necessary parameters are correctly set.

## License

BN_MANAGER is licensed under the MIT License.
