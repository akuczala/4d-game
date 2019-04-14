from Geometry import *
import pygame
import numpy as np
import vec
from colors import *
class Camera:
	ang_speed = 0.05
	speed = 0.1
	def __init__(self,pos,draw_class,angles = None):
		d = len(pos)

		self.pos = pos
		if angles is None:
			self.angles = np.zeros([d])
		else:
			self.angles = angles
		
		self.ref_frame = np.eye(d)
		self.frame = self.ref_frame
		self.clipping = True
		self.cheld = False #this is a kludge for pressing a key once
		self.update_rot_matrix(0,1,0)
	def update_rot_matrix(self,axis1,axis2,angle):
		#rows of the frame are the vectors. so to transform the frame, we multiply on the right
		R = vec.rotation_matrix(self.frame[axis1],self.frame[axis2],angle)
		self.frame = np.dot(self.frame, R)

		self.rot_matrix = self.frame.T

		self.rot_matrix_T = self.rot_matrix.T
	def look_at(self,p):
		self.frame = vec.rotation_matrix(self.ref_frame[-1],p).T
		self.rot_matrix = self.frame.T
		self.rot_matrix_T = self.rot_matrix.T
	def rotate(self,point,inverse=False):
		if not inverse:
			 #equiv to mult point on left by matrix. this form allows easy batching
			return np.dot(point,self.rot_matrix_T)
		else:
			return np.dot(point,self.rot_matrix)
	def transform(self,point):
		return self.rotate(point-self.pos,inverse=True)
		#return point - np.array([0,0,-5])
	def heading(self):
		#return self.rotate(np.array([0,0,1]),inverse=True)
		#Rxz = rotation_matrix_aligned(0,2,-self.angles[0])
		#return np.dot(self.heading_matrix_T,self.ref_frame[-1])
		return self.frame[-1]
	def draw_frame_lines(self,draw_class):
		d = len(self.pos)
		frame_origin = self.frame[-1]*0.1
		frame_origin += self.pos
		frame_lines = np.stack((np.zeros([d,d]),self.frame)).transpose(1,0,2)
		frame_lines = frame_lines*0.5 + frame_origin
		for frame_line, color in zip(frame_lines,[PURPLE,MAGENTA,ORANGE,CYAN][:d]):
			draw_class.draw_lines(self,[frame_line],color)

	def input_update(self,draw_class,events):
		update = False
		keys = pygame.key.get_pressed()
		
		#toggle clipping
		if keys[pygame.K_c]:
			if not self.cheld:
				self.clipping = not self.clipping
				self.cheld = True
				print('clipping',self.clipping)
		else:
			self.cheld = False

		key_in = False
		mouse_in = False
		if keys[pygame.K_w]:
			self.pos = self.pos + self.speed*self.heading()
			update = True
			key_in = True
		if keys[pygame.K_s]:
			self.pos = self.pos - self.speed*self.heading()
			key_in = True
			update = True

		d = len(self.pos)

		if d == 3:
			trans_keymapping = {
				pygame.K_l : (0, 1),
				pygame.K_j : (0,-1),
				pygame.K_i : (1, 1),
				pygame.K_k : (1,-1)

			}
			rot_keymapping = {
				pygame.K_a : (0, 1,-1),
				pygame.K_d : (0, 1, 1),
				pygame.K_l : (0, 2, 1),
				pygame.K_j : (0, 2,-1),
				pygame.K_i : (1, 2, 1),
				pygame.K_k : (1, 2,-1)
			}
		if d == 4:
			trans_keymapping = {
				pygame.K_a : (2,-1),
				pygame.K_d : (2, 1),
				pygame.K_l : (0, 1),
				pygame.K_j : (0,-1),
				pygame.K_i : (1, 1),
				pygame.K_k : (1,-1)

			}
			rot_keymapping = {
				pygame.K_o : (0, 2, 1),
				pygame.K_u : (0, 2,-1),
				pygame.K_a : (2, 3,-1),
				pygame.K_d : (2, 3, 1),
				pygame.K_l : (0, 3, 1),
				pygame.K_j : (0, 3,-1),
				pygame.K_i : (1, 3, 1),
				pygame.K_k : (1, 3,-1)
			}
		#holding alt translates rather than rotates the camera
		if keys[pygame.K_RALT] or keys[pygame.K_LALT]:
			for key_id in trans_keymapping:
				if keys[key_id]:
					axis, trans_sign = trans_keymapping[key_id]
					self.pos = self.pos + trans_sign*self.speed*self.frame[axis]
					update = True
		else:
			for key_id in rot_keymapping:
				if keys[key_id]:
					axis1, axis2, angle_sign = rot_keymapping[key_id]
					self.update_rot_matrix(axis1, axis2, angle_sign*self.ang_speed)
					update = True
		#mouse
		#dmx, dmy = pygame.mouse.get_rel()
		#mx, my = pygame.mouse.get_pos() #accesses state; mouse may not be moving RIGHT NOW
		#check for mouse motion events
		dmx, dmy = 0,0
		#there can be many events here. which to choose?
		for event in events:
			if event.type == pygame.MOUSEMOTION:
				#print('mooovesd',event.pos)
				mx, my = event.pos
				dmx, dmy = mx - draw_class.center[0], my - draw_class.center[1]
				#break #choose first event
		if abs(dmx) > 2 or abs(dmy) > 2:
			#print(dmx,dmy)
			update = True
			pygame.mouse.set_pos(draw_class.center)
			if d == 3:
				self.update_rot_matrix(1, 2, -dmy/draw_class.height*32*self.ang_speed)
				self.update_rot_matrix(0, 2, dmx/draw_class.height*32*self.ang_speed)
			if d == 4:
				if keys[pygame.K_RSHIFT] or keys[pygame.K_LSHIFT]:
					self.update_rot_matrix(2, 3, dmx/draw_class.height*32*self.ang_speed)
				else:
					self.update_rot_matrix(0, 3, dmx/draw_class.height*32*self.ang_speed)
				self.update_rot_matrix(1, 3, -dmy/draw_class.height*32*self.ang_speed)
			mouse_in = True
		
		
		#if key_in:
		#	print('key in')
		return update
	def update(self,draw_class,events):
		update = self.input_update(draw_class,events)
		return update
