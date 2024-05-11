'''
The harness plays as a server.
'''
import zmq
import time

def main():

	ctx = zmq.Context()
	socket = ctx.socket(zmq.REP)
	socket.bind("tcp://*:5555")

	while True:
		msg = socket.recv()
		print(msg)
		time.sleep(1)
		socket.send_string('World')
		print('The harness is waiting for the request from the rust client.')

	# pass

if __name__ == '__main__':
	main()