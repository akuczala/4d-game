import itertools
import vec
from vec import Vec
import numpy as np

from util import flatten, unique
import math
from numpy.linalg import svd
#a point is a vector
Point = Vec

#a line is a 2 tuple of points
# def Line(p1,p2):
#   return (p1,p2)


#or, a line is a class that encapsulates a 2-tuple, which comes with nice printing
#somehow this supports iteration
class Line():
    def __init__(self, p1, p2):
        self.points = (p1, p2)

    def __getitem__(self, i):
        return self.points[i]

    def __str__(self):
        return "Line(" + str(self.points[0]) + "," + str(self.points[1]) + ")"

    __repr__ = __str__


#apply function to each point in line
def line_map(f, line, *args, **kwargs):
    return Line(f(line[0], *args, **kwargs), f(line[1], *args, **kwargs))


class HyperPlane:
    def __init__(self, normal, threshold):
        self.normal = normal
        self.threshold = threshold


def line_plane_intersect(line, plane):
    p0 = line[0]
    p1 = line[1]
    n = plane.normal
    th = plane.threshold
    p0n = vec.dot(p0, n)
    p1n = vec.dot(p1, n)
    #line is contained in plane
    if vec.isclose(p0n, 0) and vec.isclose(p1n, 0):
        return None
    #plane does not intersect line segment
    t = (p0n - th) / (p0n - p1n)
    if t < 0 or t > 1:
        return None
    return vec.linterp(p0, p1, t)


# an edge is a 2-tuple of vertex indices
# def Edge(vi1,vi2):
#   return (vi1,vi2)
# an edge is a class encapsulating a 2-tuple of vertex indices
class Edge():
    def __init__(self, vi1, vi2):
        self.vertis = (vi1, vi2)

    def __getitem__(self, i):
        return self.vertis[i]

    def __str__(self):
        return "Edge(" + str(self.vertis[0]) + "," + str(self.vertis[1]) + ")"

    __repr__ = __str__


#A face is more complicated
class Face:
    def __init__(self, edgeis, normal, color=None):
        self.edgeis = edgeis  #list of edge indices belonging to some shape
        self.normal = normal  #current normal
        self.normal_ref = normal.copy(
        )  # normal's orientation relative to the shape
        if color is None:
            self.color = (255, 255, 255)  #default white
        else:
            self.color = color

    def copy(self):
        return Face(self.edgeis.copy(), self.normal_ref.copy(),
                    self.color)  #color is a tuple and therefore immutable

    def __str__(self):
        return 'Face, edgeis ' + str(self.edgeis) + ', normal_ref= ' + str(
            self.normal_ref)

    __repr__ = __str__

    #return vertex indices included in face
    #might want to run this only once, and store vert indices in face
    def get_verts(self, shape):
        return unique(flatten([shape.edges[ei] for ei in self.edgeis]))

    #rotate face normals and recalculate centers, thresholds
    def update(self, shape, rot_mat):
        self.normal = vec.dot(rot_mat, self.normal_ref)
        self.center = vec.barycenter(
            [shape.verts[vi] for vi in self.get_verts(shape)])
        self.threshold = vec.dot(self.normal, self.center)

    def update_visibility(self, camera):
        norm_dot_cam = vec.dot(self.normal, self.center - camera.pos)
        self.visible = norm_dot_cam < 0

    def check_normal(self,shape):
        vertis = self.get_verts(shape)
        verts = [shape.verts_ref[vi] for vi in vertis]
        dots = [vec.dot(v-self.center,self.normal_ref) for v in verts]
        return np.all(vec.isclose(dots,vec.zero_vec(len(dots))))


def num_planes(d):
    return d * (d - 1) // 2


class ConvexShape:
    def __init__(self, verts, edges, faces, pos=None, frame = None, scale=None):
        d = vec.dim(verts[0])
        self.verts_ref = verts  #relative vertex positions
        self.verts = verts.copy()  #absolute vertex positions
        self.edges = edges  #list of edges
        self.faces = faces  #list of faces
        self.ref_frame = vec.eye(d)
        self.transparent = False
        if pos is None:
            self.pos = vec.zero_vec(d)
        else:
            self.pos = pos
        if frame is None:
            self.frame = vec.eye(d)
        else:
            self.frame = frame
        if scale is None:
            self.scale = 1.
        else:
            self.scale = scale
        self.subfaces = self.calc_subfaces()
        self.update()
        if not self.check_normals():
            print("Warning: not all normals perpendicular")
    #full copy
    def copy(self):
        faces_copy = [face.copy() for face in self.faces]
        return ConvexShape(
            self.verts_ref,
            self.edges.copy(),
            faces_copy,
            pos=self.pos.copy(),
            frame=self.frame.copy(),
            scale=self.scale)

    def transform(self):
        #rotate and translate vertices from reference points
        self.verts = [
            vec.dot(self.frame, v * self.scale) + self.pos for v in self.verts_ref
        ]
        #update faces
        for face in self.faces:
            face.update(self, self.frame)
    #get line corresponding to edge
    def get_edge_line(self, edge):
        return [self.verts[vi] for vi in edge]
    #get line corresponding to edge index
    def get_edgei_line(self, ei):
        return self.get_edge_line(self.edges[ei])
    def rotate(self, axis1, axis2, angle):
        #rows of the frame are the vectors. so to transform the frame, we multiply on the right
        R = vec.rotation_matrix(self.frame[axis1], self.frame[axis2], angle)
        self.frame = vec.dot(self.frame, R)
        self.update()
    def calc_bounding_radius(self):
        return max(vec.dot(dv,dv) for dv in ((v - self.pos) for v in self.verts))
    #update shape
    def update(self, pos=None, frame=None, scale=None):
        if pos is not None:
            self.pos = pos
        if frame is not None:
            self.frame = frame
        if scale is not None:
            self.scale = scale
        self.transform()
        self.bounding_radius = self.calc_bounding_radius()

    #find indices of (d-1) faces that are joined by a (d-2) edge
    def calc_subfaces(self):
        d = vec.dim(self.verts[0])
        if d == 3:
            n_target = 1
        if d == 4:
            n_target = 2
        subfaces = []
        for i, j in itertools.combinations(range(len(self.faces)), 2):
            if ConvexShape.count_common_edges(self.faces[i],
                                              self.faces[j]) >= n_target:
                subfaces.append(Edge(i, j))
        return subfaces

    def count_common_edges(face1, face2):
        both_edgeis = face1.edgeis + face2.edgeis
        return len(both_edgeis) - len(set(both_edgeis))

    def update_visibility(self, camera):
        for face in self.faces:
            if self.transparent:
                face.visible = True
            else:
                face.update_visibility(camera)

    def check_normals(self):
        return np.all([face.check_normal(self) for face in self.faces])

#note: uses numpy quite, and assumes verts and edges
#are lists of numpy arrays
#these are converted into Vec and Edge objects at the end
def build_cube(d):
    verts = list(itertools.product(range(2), repeat=d))

    def calc_cube_edges(verts):
        d = vec.dim(verts[0])
        edges = []
        for i, j in itertools.combinations(range(len(verts)), 2):
            n_shared = np.count_nonzero(
                np.logical_not(np.logical_xor(verts[i], verts[j])))
            if n_shared == d - 1:
                edges.append([i, j])
        return edges

    edges = calc_cube_edges(verts)

    def calc_cube_faces(edges, verts):
        d = vec.dim(verts[0])
        faces = []
        n_face_verts = 2**(d - 1)
        faces_vi = []
        #compute indices of vertices within each face
        for i in range(d):
            faces_vi.append(
                np.sum(
                    2**np.arange(d) * np.roll(
                        verts[:n_face_verts], i, axis=-1),
                    axis=-1))
            faces_vi.append(
                np.sum(
                    2**np.arange(d) * np.roll(
                        verts[n_face_verts:], i, axis=-1),
                    axis=-1))
        #print(faces_vi)
        faces_vi = np.array(faces_vi)
        for face_vi in faces_vi:
            face_edgeis = []
            for v1, v2 in itertools.combinations(face_vi, 2):
                for ei in range(len(edges)):
                    #test if [v1,v2] is an edge
                    if np.all(np.equal([v1, v2], edges[ei])) or np.all(
                            np.equal([v2, v1], edges[ei])):
                        face_edgeis.append(ei)
            #calc face normal
            verts_in_face = np.array(verts)[face_vi]
            align_1 = np.all(verts_in_face, axis=0)
            align_0 = np.all(np.logical_not(verts_in_face), axis=0)
            face_normal = Vec(
                list(align_1.astype(np.float) - align_0.astype(np.float)))
            #print(face_normal)
            faces.append(Face(face_edgeis, face_normal))

        return faces

    faces = calc_cube_faces(edges, verts)
    #scale and translate verts so that the cube's corners are at +- 1
    #and the center is in the center
    verts = [2*Point(list(v)) - vec.ones_vec(d) for v in verts] #convert to Point data type
    edges = [Edge(e[0],e[1]) for e in edges] # convert to Edges data type
    return ConvexShape(verts, edges, faces)

#doesn't clip other shapes properly, although if our target is
#always the farthest object this doesnt matter
def build_3d_target():
    face_verts = [Vec([1,1,0]),Vec([1,-1,0]),Vec([-1,-1,0]),Vec([-1,1,0])]
    face_edges = [Edge(0,1),Edge(1,2),Edge(2,3),Edge(3,0)]
    return ConvexShape(verts = face_verts, edges = face_edges,
                                     faces = [Face([0,1,2,3],normal = Vec([0,0,-1]))])

def build_floor(d,n,scale,h):
    r = scale*n
    # if d == 3:
    #     perm = [0,2,1]
    # else:
    #     perm = [0,2,3,1]
    # def make_vert(floor_vert):
    #     vec.permute(floor_vert,perm)
    # thing = [[[scale*i,-r,h],[scale*i,r,h]] for i in range(n)]
    # lines = flatten(Line(vec.permute
    if d == 3:
        lines = flatten([
            [Line(
            Vec([scale*i,h,-r]),
            Vec([scale*i,h,r]))
        for i in range(-n,n+1)],
        [Line(
            Vec([-r,h,scale*i]),
            Vec([r,h,scale*i]))
        for i in range(-n,n+1)]
        ])
    if d == 4:
        lines = flatten([
            [Line(
            Vec([scale*i,h,scale*j,-r]),
            Vec([scale*i,h,scale*j,r]))
        for i,j in itertools.product(range(-n,n+1),range(-n,n+1))],
        [Line(
            Vec([-r,h,scale*j,scale*i]),
            Vec([r,h,scale*j,scale*i]))
        for i,j in itertools.product(range(-n,n+1),range(-n,n+1))],
        [Line(
            Vec([scale*i,h,-r,scale*j]),
            Vec([scale*i,h,r,scale*j]))
        for i,j in itertools.product(range(-n,n+1),range(-n,n+1))]
        ])
    return lines

#builds 3d cylinder
def build_cylinder(r,h,axis,n_circ_pts):
    d = 3
    circle_coords = [(lambda angle: r*Vec([math.cos(angle),math.sin(angle)]))(2*math.pi*i/n_circ_pts) for i in range(n_circ_pts)]
    #normals point halfway between each pair of circle coords
    normal_coords = [(lambda angle: Vec([math.cos(angle),math.sin(angle)]))(2*math.pi*(i+0.5)/n_circ_pts) for i in range(n_circ_pts)]
    top_verts = [vec.insert_index(p,axis,h/2) for p in circle_coords]
    bottom_verts = [vec.insert_index(p,axis,-h/2) for p in circle_coords]
    verts = top_verts + bottom_verts

    top_edges = [Edge(i,(i+1)%n_circ_pts) for i in range(n_circ_pts)]
    bottom_edges = [Edge(i + n_circ_pts,(i+1)%n_circ_pts + n_circ_pts) for i in range(n_circ_pts)]
    long_edges = [Edge(i,i+n_circ_pts) for i in range(n_circ_pts)]
    edges = top_edges + bottom_edges + long_edges

    top_face = Face(edgeis = list(range(0,n_circ_pts)), normal = vec.one_hot(d,axis))
    bottom_face = Face(edgeis = list(range(n_circ_pts,2*n_circ_pts)), normal = -vec.one_hot(d,axis))
    normal_vecs = [vec.insert_index(p,axis,0) for p in normal_coords]
    long_faces = [Face(edgeis = [i,i+n_circ_pts,2*n_circ_pts + i,2*n_circ_pts + (i+1)%n_circ_pts],
        normal = normal_vecs[i]) for i in range(n_circ_pts)]

    faces = [top_face,bottom_face] + long_faces

    return ConvexShape(verts,edges,faces)

#builds 4d duocylinder
#n_circ points is a length two list of # points around each perp circle
#rs is a list of radii of each circle
#each face is a prism. if circle 0 has m points and circle 1 has n points,
#there are m n-prisms and n m-prisms
def build_duocylinder(rs,axes,n_circ_pts):
    d = 4
    if len(unique(flatten(axes))) < d:
        raise Exception("all axes must be distinct")
    circle_coords = [
    [(lambda angle: r*Vec([math.cos(angle),math.sin(angle)]))(2*math.pi*(i+0.5)/n_pts)
        for i in range(n_pts)] for r,n_pts in zip(rs,n_circ_pts)]
    
    def make_vert(circ0,circ1,axes):
        v = vec.zero_vec(d)
        v[axes[0][0]] = circ0[0]
        v[axes[0][1]] = circ0[1]
        v[axes[1][0]] = circ1[0]
        v[axes[1][1]] = circ1[1]
        return v
    verts = [make_vert(c1,c2,axes) for c1,c2 in itertools.product(*circle_coords)]

    #we need m loops of length n and n loops of length m
    edges_1 = [Edge(j+i*n_circ_pts[1],(j+1)%n_circ_pts[1]+i*n_circ_pts[1])
        for i,j in itertools.product(range(n_circ_pts[0]),range(n_circ_pts[1]))]
    edges_2 = [Edge(j+i*n_circ_pts[1],j + ((i+1)%n_circ_pts[0])*n_circ_pts[1])
        for i,j in itertools.product(range(n_circ_pts[0]),range(n_circ_pts[1]))]

    edges = edges_1 + edges_2

    #make normals point from center of shape to center of face
    #happens to work
    def make_normal(edgeis,verts,edges):
        vertis = unique(flatten([edges[ei] for ei in edgeis]))
        verts_in_face = [verts[vi] for vi in vertis]
        center = vec.barycenter(verts_in_face)
        return vec.unit(center)
    #we need m n-prisms and n m-prisms
    def make_face1(i,n_circ_pts):
        m = n_circ_pts[0]; n = n_circ_pts[1]
        cap1_edgeis = [j + i*n for j in range(n)]
        cap2_edgeis = [j + ((i+1)%m)*n for j in range(n)]
        long_edgeis = [m*n + j + i*n for j in range(n)]
        edgeis = cap1_edgeis + cap2_edgeis + long_edgeis
        return Face(edgeis = edgeis,
            normal = make_normal(edgeis,verts,edges)
            )
    def make_face2(j,n_circ_pts):
        m = n_circ_pts[0]; n = n_circ_pts[1]
        cap1_edgeis = [m*n + j + i*n for i in range(m)]
        cap2_edgeis = [m*n + (j+1)%n + i*n for i in range(m)]
        long_edgeis = [j + i*n for i in range(m)]
        edgeis = cap1_edgeis + cap2_edgeis + long_edgeis
        return Face(edgeis = edgeis,
            normal = make_normal(edgeis,verts,edges)
            )
    faces_1 = [make_face1(i,n_circ_pts) for i in range(n_circ_pts[0])]
    faces_2 = [make_face2(j,n_circ_pts) for j in range(n_circ_pts[1])]
    faces = faces_1 + faces_2
    return ConvexShape(verts,edges,faces)


#build (polar-parameterized) 3d sphere
#we exclude theta = +/- pi, and cap off the poles with circle faces
#so basically it's a warped cylinder?
def build_sphere_3d(r,phi_pts, theta_pts):
    d = 3
    verts = [r*Vec([math.cos(ph)*math.sin(th),math.sin(ph)*math.sin(th),math.cos(th)])
    for th,ph in ((math.pi*(i+1)/(theta_pts+1),2*math.pi*j/phi_pts) for (i,j) in
        itertools.product(range(theta_pts),range(phi_pts)))]

    phi_edges = [Edge(j + phi_pts*i, (j+1)%phi_pts + phi_pts*i) for i,j in itertools.product(range(theta_pts),range(phi_pts))]
    theta_edges = [Edge(j + phi_pts*i, j + phi_pts*(i+1)) for i,j in itertools.product(range(theta_pts-1),range(phi_pts))]

    edges = phi_edges + theta_edges

    top_face = Face(edgeis = list(range(0,phi_pts)), normal = vec.one_hot(d,-1))
    bottom_face = Face(edgeis = list(range(phi_pts*(theta_pts-1),phi_pts*theta_pts)), normal = -vec.one_hot(d,-1))
    def make_middle_face(i,j):
        edgeis = [j + i*phi_pts, j + (i+1)*phi_pts,
        phi_pts*theta_pts + j + i*phi_pts, phi_pts*theta_pts + (j+1)%phi_pts + i*phi_pts]

        vertis = unique(flatten([edges[ei] for ei in edgeis]))
        verts_in_face = [verts[vi] for vi in vertis]
        center = vec.barycenter(verts_in_face)
        rel_verts = [v - center for v in verts_in_face]
        #find vector perpendicular to all vertices
        #it is the column of V corresponding to the zero singular value
        #since numpy sorts singular value from largest to smallest,
        #it is the last row in V transpose.
        perp = Vec(svd(np.array(rel_verts))[-1][-1])
        #want our vector to be pointing outwards
        if vec.dot(perp,center) < 0:
            perp = -perp
        normal = vec.unit(perp)

        return Face(edgeis = edgeis, normal = normal)
    middle_faces = [make_middle_face(i,j) for i,j in itertools.product(range(theta_pts-1),range(phi_pts))]

    faces = [top_face,bottom_face] + middle_faces
    return ConvexShape(verts,edges,faces)
