package main

import (
    "context"
	"database/sql"
	"log"
    "sync"

	_ "github.com/go-sql-driver/mysql"
	"github.com/streadway/amqp"
    "golang.org/x/sync/semaphore"
)

func failOnError(err error, msg string) {
	if err != nil {
		log.Fatalf("%s: %s", msg, err)
	}
}

func main() {
	// MySQL connection
	db, err := sql.Open("mysql", "root:example@tcp(127.0.0.1:3306)/exampledb")
	failOnError(err, "Failed to connect to MySQL")
	defer db.Close()

	// RabbitMQ connection
	conn, err := amqp.Dial("amqp://guest:guest@localhost:5672/")
	failOnError(err, "Failed to connect to RabbitMQ")
	defer conn.Close()

	ch, err := conn.Channel()
	failOnError(err, "Failed to open a channel")
	defer ch.Close()

	msgs, err := ch.Consume(
		"externally_configured_queue", // queue
		"",     // consumer
		false,  // auto-ack
		false,  // exclusive
		false,  // no-local
		false,  // no-wait
		nil,    // args
	)
	failOnError(err, "Failed to register a consumer")

    var wg sync.WaitGroup
    maxConcurrentGoroutines := 120 // Limit the number of concurrent goroutines
	sem := semaphore.NewWeighted(int64(maxConcurrentGoroutines))

	ctx := context.Background()

    go func() {
		for d := range msgs {
			sem.Acquire(ctx, 1) // Acquire a semaphore before processing a message
			wg.Add(1)
			go func(d amqp.Delivery) {
				defer wg.Done()
				defer sem.Release(1) // Release the semaphore once processing is done
				// log.Printf("Received a message: %s", d.Body)
				if _, err := db.Exec("INSERT INTO messages (content) VALUES (?)", string(d.Body)); err != nil {
					log.Printf("Error inserting message into DB: %s", err)
					d.Nack(false, true) // Optionally requeue the message
					return
				}
				// log.Printf("Message saved to database")
				d.Ack(false) // Manually acknowledge the message
			}(d)
		}
	}()

	log.Printf(" [*] Waiting for messages. To exit press CTRL+C")
	wg.Wait() // Wait for all goroutines to finish
}
