# rabbitmq-mysql
Reads from a queue in rabbitmq and saves the data to mysql

Rust with semaphore 100 consumes 1 million messages consumed in 342 seconds
Rust with semaphore 120 consumes 1 million messages consumed in 338 seconds
Go with semaphore 120 consumes 1 million messages consumed in 424 seconds
