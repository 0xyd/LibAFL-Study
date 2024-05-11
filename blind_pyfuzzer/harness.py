'''
The harness plays as a server.
'''
import zmq
import time


def harness(inp:bytes):

	mapArr = bytearray([0, 0, 0, 0, 0, 0])

	if len(inp) > 1 and inp[1] == ord(b'a'):
		mapArr[1] = 1
		if len(inp) > 2 and inp[2] == ord(b'b'):
			mapArr[2] = 1
			if len(inp) > 3 and inp[3] == ord(b'c'): 
				mapArr[3] = 1
				if len(inp) > 4 and inp[4] == ord(b'd'): 
					mapArr[4] = 1
					if len(inp) > 5 and inp[5] == ord(b'e'): 
						mapArr[5] = 1
	mapArr[0] = 1
	return mapArr


def main():

	ctx = zmq.Context()
	socket = ctx.socket(zmq.REP)
	socket.bind("tcp://*:5555")

	while True:
		msg = socket.recv()
		r = harness(msg)
		time.sleep(0.001)
		socket.send(r)
		# socket.send_string('World')
		# print('The harness is waiting for the request from the rust client.')

	# pass

if __name__ == '__main__':
	main()