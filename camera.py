import vec
from Geometry import HyperPlane

class Camera:
    ang_speed = 0.05
    speed = 0.1

    def __init__(self, pos, angles=None):
        d = len(pos)

        self.pos = pos
        if angles is None:
            self.angles = vec.zero_vec(d)
        else:
            self.angles = angles

        self.ref_frame = vec.eye(d)
        self.frame = self.ref_frame
        self.update_rot_matrix(0, 1, 0)

    def update_rot_matrix(self, axis1, axis2, angle):
        #rows of the frame are the vectors. so to transform the frame, we multiply on the right
        R = vec.rotation_matrix(self.frame[axis1], self.frame[axis2], angle)
        self.frame = vec.dot(self.frame, R)

        self.rot_matrix = self.frame.T

        self.rot_matrix_T = self.rot_matrix.T

        self.update_plane()
    def look_at(self, p):
        self.frame = vec.rotation_matrix(self.ref_frame[-1], p).T
        self.rot_matrix = self.frame.T
        self.rot_matrix_T = self.rot_matrix.T

    def rotate(self, point, inverse=False):
        if not inverse:
            #equiv to mult point on left by matrix. this form allows easy batching
            return vec.dot(point, self.rot_matrix_T)
        else:
            return vec.dot(point, self.rot_matrix)

    def transform(self, point):
        return self.rotate(point - self.pos, inverse=True)
        #return point - np.array([0,0,-5])
    def heading(self):
        #return self.rotate(np.array([0,0,1]),inverse=True)
        #Rxz = rotation_matrix_aligned(0,2,-self.angles[0])
        #return np.dot(self.heading_matrix_T,self.ref_frame[-1])
        return self.frame[-1]

    def update_plane(self):
        self.plane = HyperPlane(self.frame[-1],vec.dot(self.pos,self.frame[-1]))