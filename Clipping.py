from Geometry import Line, HyperPlane
from util import flatten
import vec
import math

#calculate the boundaries of the invisible region behind a shape
def calc_boundaries(faces, subfaces, origin):
    boundaries = []
    for subface in subfaces:
        face1 = faces[subface[0]]
        face2 = faces[subface[1]]
        #if one face is visible, the other not
        if face1.visible == (not face2.visible):
            boundary = calc_boundary(face1, face2, origin)
            boundaries.append(boundary)
    #visible faces are boundaries
    for face in faces:
        if face.visible:
            boundaries.append(
                HyperPlane(normal=face.normal, threshold=face.threshold))
    return boundaries


def calc_boundary(face1, face2, origin):

    n1 = face1.normal
    n2 = face2.normal
    th1 = face1.threshold
    th2 = face2.threshold

    #k1 and k2 must be opposite signs
    k1 = vec.dot(n1, origin) - th1
    k2 = vec.dot(n2, origin) - th2

    t = k1 / (k1 - k2)

    n3 = vec.linterp(n1, n2, t)
    th3 = vec.linterp(th1, th2, t)

    return HyperPlane(n3, th3)


#returns boolean (True,False) if point is (clipped,not clipped)
def point_clipped(point, boundaries,dth=0):
    for boundary in boundaries:
        if vec.dot(point, boundary.normal) >= boundary.threshold+dth:
            return False
    return True


# argument 'lines' is modified in the body here. is this ok?
def clip_lines(lines, shape, clipping_shapes):
    #loop over shapes to check for clipping
    for clipping_shape in clipping_shapes:

        if clipping_shape is not shape and (not clipping_shape.transparent):
            clipped_lines = flatten(
                [clip_line(line, clipping_shape.boundaries) for line in lines])
            lines = clipped_lines
        else:
            clipped_lines = lines
    return clipped_lines


#return list of new, clipped lines from the set of boundaries generated from a shape
#(a single line could be clipped into 0, 1 or 2 lines)
def clip_line(line, boundaries):
    p0 = line[0]
    p1 = line[1]

    a = 0.
    b = 1.
    p0_all_safe, p1_all_safe = False, False

    for boundary in boundaries:

        n = boundary.normal
        th = boundary.threshold

        p0n = vec.dot(p0, n)
        p1n = vec.dot(p1, n)

        p0_safe = p0n >= th
        p1_safe = p1n >= th

        if p0_safe and p1_safe:
            a = 0
            b = 1
            p0_all_safe = True
            p1_all_safe = True
            break
        #print('p0,p1 safe',p0_safe,p1_safe)
        if p0_safe and (not p1_safe):
            t_intersect = (p0n - th) / (p0n - p1n)
            a = max(a, t_intersect)
            #print('move a to',a)
        if (not p0_safe) and p1_safe:
            t_intersect = (p0n - th) / (p0n - p1n)
            b = min(t_intersect, b)
            #print('move b to',b)
        p0_all_safe = (p0_all_safe or p0_safe)
        p1_all_safe = (p1_all_safe or p1_safe)

    #print('all_safe',p0_all_safe,p1_all_safe)
    #both endpoints visible
    if p0_all_safe and p1_all_safe:
        #return two lines if we've intersected the shape
        if a > 0 and b < 1:
            return [
                Line(p0, vec.linterp(p0, p1, a)),
                Line(vec.linterp(p0, p1, b), p1)
            ]
        else:
            #return entire line if we haven't intersected the shape
            return [line]
    if p0_all_safe and (not p1_all_safe):
        return [Line(p0, vec.linterp(p0, p1, a))]
    if (not p0_all_safe) and p1_all_safe:
        return [Line(vec.linterp(p0, p1, b), p1)]
    #if neither point is visible, don't draw the line
    return []


#clip everything behind the plane at x[-1] = small_z
def clip_line_z0(line, z0, small_z):
    v1 = line[0]
    v2 = line[1]
    #if both vertices are behind, draw neither
    if v1[-1] <= z0 and v2[-1] <= z0:
        return None
    #both vertices in front
    if v1[-1] > z0 and v2[-1] > z0:
        return line
    #if one of the vertices is behind the camera
    intersect = plane0_intersect(v1, v2, z0 + small_z)
    if v1[-1] < 0 and v2[-1] > 0:
        return [intersect, v2]
    else:
        return [v1, intersect]

#clip everything behind the plane
def clip_line_plane(line, plane, small_z=0):
    p0 = line[0]
    p1 = line[1]

    n = plane.normal
    th = plane.threshold +small_z

    p0n = vec.dot(p0, n)
    p1n = vec.dot(p1, n)

    p0_safe = p0n >= th
    p1_safe = p1n >= th
    #if both vertices are behind, draw neither
    if (not p0_safe) and (not p1_safe):
        return None
    #both vertices in front
    if p0_safe and p1_safe:
        return line
    #if one of the vertices is behind the camera
    t_intersect = (p0n - th ) / (p0n - p1n)
    intersect = vec.linterp(p0,p1,t_intersect)
    if (not p0_safe) and p1_safe:
        return Line(intersect, p1)
    else:
        return Line(p0, intersect)

def plane0_intersect(v1, v2,
                     z0):  #point of intersection with plane at x[-1] = z0
    t = (v1[-1] - z0) / (v1[-1] - v2[-1])
    return vec.linterp(v1, v2, t)

#INCOMPLETE / should probably use existing boundary routines
#clip line to lie within a cube at the origin of radius r
#returns clipped line
def clip_line_cube(line, r):
    v0 = line[0]
    v1 = line[1]
    v0_in_cube = vec.linf_norm(v0) < r
    v1_in_cube = vec.linf_norm(v1) < r
    #within cube
    if v0_in_cube and v1_in_cube:
        return line
    #outside cube
    if (not v0_in_cube) and (not v1_in_cube):
        return None

    #need to determine which plane of the cube
    #that the line intersects
    # if (not v0_in_cube) and v1_in_cube:
    #     return Line(intersect, v1)
    # else:
    #     return Line(v0, intersect)

#returns either none or pair of intersecting points
#note that tm and p are NOT bound between 0 and 1
def sphere_line_intersect(line, r):

    v0 = line[0]
    v1 = line[1]
    dv = v1 - v0
    dv_norm = vec.norm(dv)
    dv = dv / dv_norm

    #in our case, sphere center is the origin
    v0_rel = v0  # - sphere_center
    v0r_dv = vec.dot(v0_rel, dv)

    discr = (v0r_dv)**2 - vec.dot(v0_rel, v0_rel) + r * r

    #print('discr',discr)
    #no intersection with line
    if discr < 0:
        return None

    sqrt_discr = math.sqrt(discr)
    tm = -v0r_dv - sqrt_discr
    tp = -v0r_dv + sqrt_discr

    #print('tm,tp',tm,tp)
    #no intersection with line segment
    if tm > dv_norm and tp > dv_norm:
        return None
    if tm < 0 and tp < 0:
        return None
    intersect_points = [v0 + tm * dv, v0 + tp * dv]
    
    return intersect_points

def sphere_t_intersect(line,r):
    v0 = line[0]
    v1 = line[1]
    v0sq = vec.dot(v0,v0)
    v1sq = vec.dot(v1,v1)
    v0dotv1 = vec.dot(v0,v1)
    #quadratic parameters
    a = v0sq + v1sq - 2*v0dotv1
    b = 2*(-v0sq + v0dotv1)
    c = v0sq - r*r

    discr = b*b-4*a*c

    if discr < 0:
        return None

    sqrt_discr = math.sqrt(discr)
    tm = (-b - sqrt_discr)/(2*a)
    tp = (-b + sqrt_discr)/(2*a)

    return (tm,tp)

def clip_line_sphere(line, r):
    v0 = line[0]
    v1 = line[1]

    v0_in_sphere = vec.dot(v0, v0) < r * r
    v1_in_sphere = vec.dot(v1, v1) < r * r

    #print('v0_in_sphere',v0_in_sphere)
    #print('v1_in_sphere',v1_in_sphere)
    if v0_in_sphere and v1_in_sphere:
        return line
    intersect = sphere_line_intersect(line, r)
    if intersect is None:
        return None
    if (not v0_in_sphere) and (not v1_in_sphere):
        return intersect
    if (not v0_in_sphere) and v1_in_sphere:
        return Line(intersect[0], v1)
    else:
        return Line(v0, intersect[1])

#axis coordinate is not properly clipped by tube
#need t value at intersection
def clip_line_cylinder(line,r,h,axis):
    def make_line(u0,u1,a0,a1,axis):
        return Line(vec.insert_index(u0,axis,a0),vec.insert_index(u1,axis,a1))

    v0 = line[0]
    v1 = line[1]
    #components parallel to axis
    a0 = v0[axis]
    a1 = v1[axis]
    #components perpendicular to axis
    u0 = vec.drop_index(v0,axis)
    u1 = vec.drop_index(v1,axis)
    #line is outside
    if (a0 > h and a1 > h) or (a0 < -h and a1 < -h):
        return None

    #clip lines to be within cylinder radius
    #would be nice if we could neatly merge this functionality with sphere clipping
    t_roots = sphere_t_intersect(Line(u0,u1),r)
    u0_in_circle = vec.dot(u0, u0) < r * r
    u1_in_circle = vec.dot(u1, u1) < r * r

    if u0_in_circle and u1_in_circle:
        circ_line = Line(u0,u1)
    else:
        if (not u0_in_circle) and (not u1_in_circle):
            #(extended) line does not intersect circle
            if t_roots is None:
                return None
            #check to see if segment intersecs
            #need only check one root to see if line segment intersects
            if t_roots[0] < 0 or t_roots[0] > 1:
                return None
            else:
                circ_line = Line(vec.linterp(u0,u1,t_roots[0]),vec.linterp(u0,u1,t_roots[1]))
                new_a0 = vec.linterp(a0,a1,t_roots[0])
                new_a1 = vec.linterp(a0,a1,t_roots[1])
                a0 = new_a0; a1 = new_a1
        else:
            if u0_in_circle and (not u1_in_circle):
                #print("u0 in",tm,tp,line)
                circ_line = Line(u0,vec.linterp(u0,u1,t_roots[1]))
                a1 = vec.linterp(a0,a1,t_roots[1])
            else:
                #print("u1 in",tm,tp,line)
                circ_line = Line(vec.linterp(u0,u1,t_roots[0]),u1)
                a0 = vec.linterp(a0,a1,t_roots[0])

    new_line = make_line(circ_line[0],circ_line[1],a0,a1,axis)
    
    #clip top and bottom of cylinder
    d = vec.dim(v0)
    clipped_line = clip_line_plane(new_line,HyperPlane(-vec.one_hot(d,axis),-h))
    if clipped_line is None:
        return None

    clipped_line = clip_line_plane(clipped_line,HyperPlane(vec.one_hot(d,axis),-h))

    return clipped_line