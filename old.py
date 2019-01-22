size = width, height = 800, 600
center = np.array([width,height])/2
BLACK = 0, 0, 0
GREEN = 0, 255, 0
RED = 255, 0 , 0

#and NOT a numpy array for updates to work


square = np.array([[1,1,0],[-1,1,0],[-1,-1,0],[1,-1,0]])
square1 = square + np.array([0,0,-1])
square2 = square + np.array([0,0,1])
connect = np.array([[[1,1,-1],[1,1,1]],[[-1,1,1],[-1,1,-1]],[[-1,-1,-1],[-1,-1,1]],[[1,-1,-1],[1,-1,1]]])
cube_edges = np.concatenate((np.concatenate(list(map(vert_list_to_edges,[square1,square2]))),connect))

test_simplex = Simplex(np.array([[1,0,0],[0,1,0],[1,1,0]]),[True,True,True],1,GREEN)

convex_simverts = np.array([
    [[1,0,0],[0,1,0],[1,1,0]],
    [[1,0,0],[0,1,0],[0,0,0]]
                           ])
convex_signs = [1,-1]
convex_
test_convex_simplices = list(map(
    lambda verts, sign: Simplex(verts,[True,True,True],sign,GREEN),
    convex_simverts,convex_signs))
test_convex = ConvexShape(np.array([0,0,0]),np.array([0,0,0]),test_convex_simplices)

class ConvexShape:
    def __init__(self,pos,angles,simplices_mesh):
        self.pos = pos
        self.angles = angles
        self.simplices_mesh = simplices_mesh
        self.simplices = simplices_mesh
        self.update()
    def transform_simplices(self):
        rot_mat = rotation_matrix(np.array([1,0,0]),np.array([0,0,1]),self.angles[0])
        self.simplices = [sm.get_transformed_simplex(self.pos,rot_mat) for sm in self.simplices_mesh]
            
    def update(self):
        self.transform_simplices()
    def draw(self,camera):
        for simplex in self.simplices:
            simplex.draw(camera)
class Simplex:
    def __init__(self,verts,drawn_edges,normal_sign,color):
        self.verts = verts
        self.drawn_edges = drawn_edges
        self.normal_sign = normal_sign
        self.update_normal()
        self.update_center()
        self.color = color
    def update_center(self):
        self.center = np.mean(verts,axis=0)
    def update_normal(self):
        verts = self.verts
        sign = self.normal_sign
        
        dverts = (verts - np.roll(verts,1,axis=0))[:-1] #take first N-1 vectors
        n = sign*np.cross(dverts[0],dverts[1])
        self.normal = n/lin.norm(n)
    def get_transformed_simplex(self,pos,rot_mat):
        verts = list(map(lambda v: np.dot(v,rot_mat) + pos,self.verts))
        return Simplex(verts,self.drawn_edges,self.normal_sign,color)
    def draw(self,camera):
        norm_dot_cam = np.dot(self.normal,self.center - camera.pos)
        if norm_dot_cam < 0:
            #draw face
            camera.draw_face(verts,self.color)
            #draw edges
            edges_to_draw =  vert_list_to_edges(self.verts)[self.drawn_edges]
            for edge in self.get_visible_edges(camera):
                camera.draw_edge(edge,self.color)

def edges_to_verts(edges):
    dim = edges.shape[-1]
    return np.unique(cube_edges.reshape(-1,dim),axis=0)



    #     def draw_line(self,line,color):
#         line_rel = np.array(list(map(self.transform,line)))
#         clipped_line = Clipping.clip_line_z0(line_rel,small_z)
#         #only draw if z > 0
#         if clipped_line[0,-1] >= 0 and clipped_line[1,-1] >= 0:
#             points = np.vectorize(proj_2d,signature='(n)->(m)')(clipped_line)
#             #points = points*100 + center
#             try:
#                 self.draw_class.draw_line_2d(points,color)
#             except:
#                 print('problem drawing',points)



# square = np.array([[1,1,0],[-1,1,0],[-1,-1,0],[1,-1,0]])
# square1 = square + np.array([0,0,-1])
# square2 = square + np.array([0,0,1])
# connect = np.array([[[1,1,-1],[1,1,1]],[[-1,1,1],[-1,1,-1]],[[-1,-1,-1],[-1,-1,1]],[[1,-1,-1],[1,-1,1]]])
# cube_lines_test = 2*np.concatenate((np.concatenate(list(map(vert_list_to_lines,[square1,square2]))),connect))
# axes_lines = 2*np.array([
#     [[1,0,0],[-1,0,0]],
#     [[0,1,0],[0,-1,0]],
#     [[0,0,1],[0,0,-1]]])
# cube_verts = np.array(list(itertools.product(range(2), repeat=3)))
# cube_verts = cube_verts - 0.5*np.ones([3])
# cube_edges = np.array([[0,1],[0,2],[0,4],[1,3],[1,5],[2,3],[2,6],[3,7],[4,5],[4,6],[5,7],[6,7]],dtype=np.int)
# cube_face_edges = np.array([[0,3,5,1],[0,4,8,2],[1,6,9,2],[11,9,8,10],[11,6,5,7],[10,4,3,7]],dtype=np.int)
# cube_normals = -np.array([[1,0,0],[0,1,0],[0,0,1],[-1,0,0],[0,-1,0],[0,0,-1]],dtype=np.float)


def vert_list_to_lines(verts,cycle=True):
    cycle_edges = np.stack((verts,np.roll(verts,1,axis=0))).transpose([1,0,2])
    if cycle and len(verts) > 2:
        return cycle_edges
    else:
        return cycle_edges[:-1]

