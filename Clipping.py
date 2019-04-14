import numpy as np
from Geometry import *
import vec
#calculate the boundaries of the invisible region behind a shape
def calc_boundaries(faces,subfaces,origin):
	boundaries = []
	for subface in subfaces:
		face1 = faces[subface[0]]
		face2 = faces[subface[1]]
		#if one face is visible, the other not
		if face1.visible == (not face2.visible):
			boundary = calc_boundary(face1,face2,origin)
			boundaries.append(boundary)
	#visible faces are boundaries
	for face in faces:
		if face.visible:
			boundaries.append(HyperPlane(normal = face.normal, threshold = face.threshold))
	return boundaries

def calc_boundary(face1,face2,origin):
	#print('-----')

	n1 = face1.normal
	n2 = face2.normal
	th1 = face1.threshold
	th2 = face2.threshold

	#print('n1',n1,'n2',n2)
	#print('th1',th1,'th2',th2)
	#k1 and k2 must be opposite signs
	k1 = vec.dot(n1,origin) - th1
	k2 = vec.dot(n2,origin) - th2

	#print('k1',k1,'k2',k2)
	t = k1/(k1-k2)

	n3 = (1-t)*n1 + t*n2
	th3 = (1-t)*th1 + t*th2

	#print('n3',n3)
	#print('th3',th3)

	return HyperPlane(n3,th3)

#returns boolean (True,False) if point is (clipped,not clipped)
def point_clipped(point,boundaries):
	for boundary in boundaries:
		if vec.dot(point,boundary.normal) >= boundary.threshold:
			return False
	return True
def clip_lines(lines,shape,clipping_shapes):
	#loop over shapes to check for clipping
    for clipping_shape in clipping_shapes:
        
        #print('clipping shape: ' + clipping_shape.name)
        
        if clipping_shape is not shape and (not clipping_shape.transparent):
            #clipped_lines = [np.reshape([],(0,lines.shape[1],lines.shape[-1]))]
            clipped_lines = []
            for line in lines:
                new_lines = clip_line(line,clipping_shape.boundaries)
                if new_lines is not None:
                    #clipped_lines = np.concatenate((clipped_lines,new_lines))
                    clipped_lines.append(new_lines)
                    #print(clipped_lines)
            lines = clipped_lines
        else:
        	clipped_lines = lines
    return clipped_lines
#return new, clipped line from the set of boundaries generated from a shape
def clip_line(line,boundaries):
	p0 = line[0]; p1 = line[1]

	a = 0.; b= 1.
	p0_all_safe, p1_all_safe = False, False

	for boundary in boundaries:

		n = boundary.normal
		th = boundary.threshold

		p0n = vec.dot(p0,n)
		p1n = vec.dot(p1,n)

		p0_safe = p0n >= th
		p1_safe = p1n >= th

		if p0_safe and p1_safe:
			a = 0; b=1;
			p0_all_safe = True;
			p1_all_safe = True
			break
		#print('p0,p1 safe',p0_safe,p1_safe)
		if p0_safe and (not p1_safe):
			t_intersect = (p0n-th)/(p0n-p1n)
			a = max(a,t_intersect)
			#print('move a to',a)
		if (not p0_safe) and p1_safe:
			t_intersect = (p0n-th)/(p0n-p1n)
			b = min(t_intersect,b)
			#print('move b to',b)
		p0_all_safe = (p0_all_safe or p0_safe)
		p1_all_safe = (p1_all_safe or p1_safe)


	#print('all_safe',p0_all_safe,p1_all_safe)
	#both endpoints visible
	if p0_all_safe and p1_all_safe:
		#return two lines if we've intersected the shape
		if a > 0 and b < 1:
			return [[p0,(1-a)*p0 + a*p1],[(1-b)*p0 + b*p1,p1]]
		else:
			#return entire line if we haven't intersected the shape
			return [line]
	if p0_all_safe and (not p1_all_safe):
		return [[p0,(1-a)*p0 + a*p1]]
	if (not p0_all_safe) and p1_all_safe:
		return [[(1-b)*p0 + b*p1,p1]]
	#if neither point is visible, don't draw the line
	return None

#clip everything behind the plane at x[-1] = small_z
def clip_line_z0(line,z0,small_z):
    v1 = line[0]; v2 = line[1]
    #if both vertices are behind, draw neither 
    if v1[-1] <= z0 and v2[-1] <= z0:
        #return np.array([[]])
        return None
    #both vertices in front
    if v1[-1] > z0 and v2[-1] > z0:
        return line
    #if one of the vertices is behind the camera
    intersect = plane0_intersect(v1,v2,z0 + small_z)
    if v1[-1] < 0 and v2[-1] > 0:
        return [intersect,v2]
    else:
        return [v1,intersect]
def plane0_intersect(v1,v2,z0): #point of intersection with plane at x[-1] = z0
    t = (v1[-1]-z0)/(v1[-1]-v2[-1])
    return (1-t)*v1 + t*v2

#clip line to lie within a cube at the origin of radius r
#returns clipped line
def clip_line_cube(line,r):
	v0 = line[0]; v1 = line[1]
	v0_in_cube = vec.l1_norm(v0) < r
	v1_in_cube = vec.l1_norm(v1) < r
	#within cube
	if v0_in_cube and v1_in_cube:
			return line
	#outside cube
	if (not v0_in_cube) and (not v1_in_cube):
		#return np.ones([2,len(v0)])*2*r #should return nothing
		return None

	if (not v0_in_cube) and v1_in_cube:
		#wrong intersect = v0/np.max(np.abs(v0))
		return Line(intersect,v1)
	else:
		#wrong intersect = v1/np.max(np.abs(v1))
		return Line(v0,intersect)

def sphere_line_intersect(line,r):
	v0 = line[0]; v1 = line[1]
	dv = v1 - v0
	dv_norm = vec.norm(dv)
	dv = dv/dv_norm

	#in our case, sphere center is the origin
	v0_rel = v0 # - sphere_center
	v0r_dv = vec.dot(v0_rel,dv)

	discr = (v0r_dv)**2 - vec.dot(v0_rel,v0_rel) +r*r

	#print('discr',discr)
	#no intersection with line
	if discr < 0:
		return None

	sqrt_discr = np.sqrt(discr)
	tm = -v0r_dv - sqrt_discr
	tp = -v0r_dv + sqrt_discr

	#print('tm,tp',tm,tp)
	#no intersection with line segment
	if tm > dv_norm and tp > dv_norm:
		return None
	if tm < 0 and tp < 0:
		return None
	new_line =  Line(v0 + tm*dv,v0 + tp*dv)
	#print(new_line)
	return new_line
def clip_line_sphere(line,r):
	v0 = line[0]; v1 = line[1]

	v0_in_sphere = vec.dot(v0,v0) < r*r
	v1_in_sphere = vec.dot(v1,v1) < r*r
	
	#print('v0_in_sphere',v0_in_sphere)
	#print('v1_in_sphere',v1_in_sphere)
	if v0_in_sphere and v1_in_sphere:
		return line
	intersect = sphere_line_intersect(line,r)
	if intersect is None:
		return None
	if (not v0_in_sphere) and (not v1_in_sphere):
		return intersect
	if (not v0_in_sphere) and v1_in_sphere:
		return Line(intersect[0],v1)
	else:
		return Line(v0,intersect[1])


