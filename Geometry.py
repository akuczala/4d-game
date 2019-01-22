import numpy as np
import numpy.linalg as lin
import itertools
from camera import Camera

def rotmat(t):
	return np.array([[np.cos(t),np.sin(t)],[-np.sin(t),np.cos(t)]])
def rotation_matrix(v1,v2,th = None): #finds rotation matrix between two (normalized) vectors (rotates v1 to v2)
	u = v1/lin.norm(v1)
	v = v2/lin.norm(v2)
	if th is None:
		costh = np.dot(u,v)
		sinth = np.sqrt(1-costh**2)
	else:
		costh = np.cos(th)
		sinth = np.sin(th)
	#sinth = np.sin(np.arccos(costh))
	R = np.array([[costh,-sinth],[sinth,costh]])
	w = (v - np.dot(u,v)*u); w = w/lin.norm(w)
	uw_mat = np.array([u,w])
	return np.eye(len(u)) - np.outer(u,u) - np.outer(w,w) + np.dot(uw_mat.T,np.dot(R,uw_mat))
def rotation_matrix_aligned(dir1,dir2,th):
	costh = np.cos(th)
	sinth = np.sin(th)
	if dir1 == 0 and dir2 == 2:
		return np.array([[costh,0,-sinth],[0,1,0],[sinth,0,costh]])
	if dir1 == 1 and dir2 == 2:
		return np.array([[1,0,0],[0,costh,-sinth],[0,sinth,costh]])
class HyperPlane:
	def __init__(self,normal,threshold):
		self.normal = normal
		self.threshold = threshold

def line_interpolate(t,v1,v2):
	return (1-t)*v1 + t*v2

def line_plane_intersect(line,plane):
	p0 = line[0]; p1 = line[1]
	n = plane.normal
	th = plane.threshold
	p0n = np.dot(p0,n)
	p1n = np.dot(p1,n)
	#line is contained in plane
	if np.isclose(p0n,0) and np.isclose(p1n,0):
		return None
	#plane does not intersect line segment
	t = (p0n-th)/(p0n-p1n)
	if t < 0 or t > 1:
		return None
	return (1-t)*p0 + t*p1

class Face:
	def __init__(self,edges,normal,color = None):
		self.edges = edges
		self.normal = normal
		self.normal_ref = normal.copy()
		if color is None:
			self.color = (255,255,255) #default white
		else:
			self.color = color
	def copy(self):
		return Face(self.edges.copy(),self.normal_ref.copy(),self.color) #color is a tuple and therefore immutable
	def __str__(self):
		return 'Face, edges ' + str(edges) + ', normal_ref= ' + str(normal_ref)
	#return vertex indices included in face
	def get_verts(self,shape):
		return np.unique(shape.edges[self.edges].reshape(-1).astype(np.int))
	#rotate face normals and recalculate centers, thresholds
	def update(self,shape,rot_mat):
		self.normal = np.dot(rot_mat,self.normal_ref)
		self.center = np.mean(shape.verts[self.get_verts(shape)],axis=0)
		self.threshold = np.dot(self.normal,self.center)
	def update_visibility(self,camera):
		norm_dot_cam = np.dot(self.normal,self.center - camera.pos)
		self.visible = norm_dot_cam < 0

def num_planes(d):
	return d*(d-1)//2

class ConvexShape:
	def __init__(self,verts,edges,faces,pos = None,angles = None,scale=None):
		d = verts.shape[-1]
		self.verts_ref = verts
		self.verts = verts.copy()
		self.edges = edges
		self.faces = faces
		self.ref_frame = np.eye(d)
		self.transparent = False
		if pos is None:
			self.pos = np.zeros([d])
		else:
			self.pos = pos
		if angles is None:
			self.angles = np.zeros([num_planes(d)])
		else:
			self.angles = angles
		if scale is None:
			self.scale = 1.
		else:
			self.scale = scale
		self.calc_subfaces()
		self.update()
	#full copy
	def copy(self):
		faces_copy = [face.copy() for face in self.faces]
		return ConvexShape(self.verts_ref,self.edges.copy(),faces_copy,
						   pos = self.pos.copy(),angles = self.angles.copy(),scale = self.scale)
	def transform(self):
		#rotate and translate vertices from reference points
		Rxz = rotation_matrix(self.ref_frame[0],self.ref_frame[-1],self.angles[0])
		Rzy = rotation_matrix(self.ref_frame[1],self.ref_frame[-1],self.angles[1])
		rot_mat = np.dot(Rxz,Rzy)
		self.verts = np.vectorize(lambda v: np.dot(rot_mat,v*self.scale) + self.pos,
								  signature='(n)->(n)')(self.verts_ref)
		#update faces
		for face in self.faces:
			face.update(self,rot_mat)
	#get line corresponding to edge
	def get_line(self,edge):
		return self.verts[self.edges[edge]]
	#update shape
	def update(self,pos=None,angles=None,scale=None):
		if pos is not None:
			self.pos = pos
		if angles is not None:
			self.angles = angles
		if scale is not None:
			self.scale = scale
		self.transform()
	#find indices of (d-1) faces that are joined by a (d-2) edge
	def calc_subfaces(self):
		d = self.verts.shape[-1]
		if d == 3:
			n_target = 1
		if d == 4:
			n_target = 2
		subfaces = []
		for i,j in itertools.combinations(range(len(self.faces)),2):
				if ConvexShape.count_common_edges(self.faces[i],self.faces[j]) >= n_target:
					subfaces.append([i,j])
		self.subfaces = np.array(subfaces)
	def count_common_edges(face1,face2):
		both_edges = np.append(face1.edges,face2.edges)
		return len(both_edges) - len(np.unique(both_edges))
	def update_visibility(self,camera):
		for face in self.faces:
			if self.transparent:
				face.visible = True
			else:
				face.update_visibility(camera)

def build_cube(d):
	verts = np.array(list(itertools.product(range(2), repeat=d)))
	def calc_cube_edges(verts):
		d = verts.shape[-1]
		edges = np.array([],dtype=np.int).reshape(0,2)
		for i,j in itertools.combinations(range(len(verts)),2):
			n_shared = np.count_nonzero(np.logical_not(np.logical_xor(verts[i],verts[j])))
			if n_shared == d-1:
				edges = np.append(edges,[[i,j]],axis=0)
		return edges
	edges = calc_cube_edges(verts)
	def vert_to_index(vert):
		d = len(vert)
		return np.sum(2**np.flipud(np.arange(3))*vert)
	def calc_cube_faces(edges,verts):
		d = verts.shape[-1]
		faces = []
		n_face_verts = 2**(d-1)
		faces_vi = []
		#compute indices of vertices within each face
		for i in range(d):
			faces_vi.append(np.sum(2**np.arange(d)*np.roll(verts[:n_face_verts],i,axis=-1),axis=-1))
			faces_vi.append(np.sum(2**np.arange(d)*np.roll(verts[n_face_verts:],i,axis=-1),axis=-1))
		faces_vi = np.array(faces_vi)
		for face_vi in faces_vi:
			face_edges = []
			for v1, v2 in itertools.combinations(face_vi,2):
				for ei in range(len(edges)):
					#test if [v1,v2] is an edge
					if np.all(np.equal([v1,v2],edges[ei])) or np.all(np.equal([v2,v1],edges[ei])):
						face_edges.append(ei)
			#calc face normal
			align_1 = np.all(verts[face_vi],axis=0)
			align_0 = np.all(np.logical_not(verts[face_vi]),axis=0)
			face_normal = align_1.astype(np.float) - align_0.astype(np.float)
			#print(face_normal)
			faces.append(Face(face_edges,face_normal))
#         for eis in np.array(list(itertools.combinations(range(len(edges)),n_face_edges))):
#             cur_verts = verts[np.unique(edges[eis].reshape(-1))]
			
#             #check if all vertices either have all zeros or all ones in one of the coordinates
#             align_1 = np.all(cur_verts,axis=0)
#             align_0 = np.all(np.logical_not(cur_verts),axis=0)
#             aligned = np.logical_or(align_0,align_1)
					
#             axis = np.arange(d)[aligned]
#             is_face = np.any(aligned)
#             if is_face:
#                 normal = align_1.astype(np.int) - align_0.astype(np.int)
#                 faces.append(Face(eis,normal))
		return faces
	faces = calc_cube_faces(edges,verts)
	#scale and translate verts so that the cube's corners are at +- 1 and the center is in the center
	return ConvexShape(verts*2 - np.ones([d]),edges,faces)