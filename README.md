# rabbitmq-mysql
Reads from a queue in rabbitmq and saves the data to mysql. 

The bench.py script publishes 1 million messages on the queue ready to be read. 

Some results from the first save in db to the last save:

* Rust with semaphore 100 consumes 1 million messages consumed in 342 seconds
* Rust with semaphore 120 consumes 1 million messages consumed in 338 seconds
* Go with semaphore 120 consumes 1 million messages consumed in 424 seconds
