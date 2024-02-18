import pika
import time

# RabbitMQ connection parameters
parameters = pika.ConnectionParameters('localhost', 5672, '/', pika.PlainCredentials('guest', 'guest'))
# Establishing connection and channel
connection = pika.BlockingConnection(parameters)
channel = connection.channel()

connection = pika.BlockingConnection(pika.ConnectionParameters('localhost'))
channel = connection.channel()

queue_name = 'externally_configured_queue'
# Declare the queue (ensure it exists)
channel.queue_declare(queue=queue_name, durable=True)

start_time = time.time()
print("Started publishing")

# Publish messages
for i in range(1_000_000):
    message = f'Message {i}'
    channel.basic_publish(exchange='',
                          routing_key=queue_name,
                          body=message)

print("Published 1 million messages in %s seconds." % (time.time() - start_time))

connection.close()
