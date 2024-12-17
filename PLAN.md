# Daisy: SSH daisy chain exhibit

Daisy is a program which will form a massive SSH daisy chain and let the
user send messages through the chain which will eventually end up back
at their local machine.

Along the way, each link in the chain will report back to the sender
directly as soon as a message is received, allowing for a real-time* map
of the messages to be displayed.

\* Of course there will be some network latency, but it's as close to
real-time as possible.

## Architecture

Daisy will be made of the following parts:

1. Sender interface.

   This is the main interactive part of Daisy. This is where the user
   will establish the chain and send.

2. Relay link program.

   This is the program that will be executed on each link in the chain,
   and will forward messages and report message progress to the map.

4. Map.

   The map will display a map of the computers in the chain,
   highlighting the trail each message leaves along the chain.

   The map will listen for reports from the chain links as UDP packets,
   and will display those on a visual map display.

## Terminology

- **sender**: The computer from which messages are sent.

- **link**: A computer in the chain which forwards messages to other
links, and is not the controller or the destination.

- **destination**: The computer which will receive messages sent through
  the chain. Can be the same as the sender.

<!-- vim: set tw=70 : -->
