# CronCat App module

The CronCat app module is used to automate abstract account actions or other modules

# Features
- This module heavily rely on [croncat] contracts for automating actions
- Create task which contains:
  - Onchain actions
  - How frequent actions should be executed
  - It may contain: Boundary, "If this then that"... For more details on "tasks" refer to [croncat]
- Remove task
- Refill task
- It uses Abstract's account balance for creation and refilling a task 

[croncat]: https://github.com/CronCats/cw-croncat