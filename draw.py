# import matrix44
# import vector3
from Geometry import *
import Clipping
from camera import Camera
from colors import *
import numpy as np
import pygame
from OpenGL.GL import *
from OpenGL.GLU import *
from util import *
import math

small_z = 0.001
z0 = 0

#focal = 4.


def to_screen(v2, scale, center):
    return v2 * scale + center


class Draw:
    def __init__(self, pygame, size, draw_origin, focal, view_radius=5):
        self.width, self.height = self.size = size
        self.center = (self.width // 2, self.height // 2)
        self.pygame = pygame
        self.focal = focal
        self.draw_origin = draw_origin
        self.view_radius = view_radius
        #scale faces very slightly to avoid drawing edges on top of each other
        #also, for some reason, setting this to one leads to a divide by 0 error
        #when there's transparency
        self.face_scales = [0.95]

    #project d-dimensional vector to d-1 dimensional vector
    def project(self, v):
        if math.isclose(v[-1], z0):
            z = z0 + small_z
        else:
            z = v[-1]
        return self.focal * v[:-1] / (z)

    def draw(self, camera, shapes):

        self.init_draw()

        for shape in shapes:
            shape.update_visibility(camera)
            shape.boundaries = Clipping.calc_boundaries(
                shape.faces, shape.subfaces, camera.pos)

        for shape in shapes:

            for face in shape.faces:
                if face.visible:
                    color = face.color

                    #calculate scaled lines, for aesthetics
                    def scale_point(p, scale):
                        return vec.linterp(face.center, p, scale)

                    #self.draw_face_fuzz(camera,face,shape,shapes)
                    lines = [shape.get_edgei_line(ei) for ei in face.edgeis]

                    scaled_lines = flatten(
                        [[line_map(scale_point, p, scale_face) for p in lines]
                         for scale_face in self.face_scales])
                    lines = scaled_lines

                    if camera.clipping:
                        clipped_lines = Clipping.clip_lines(
                            lines, shape, shapes)
                        #draw clipped line
                        self.draw_lines(camera, clipped_lines, color)
                    else:  #noclip
                        self.draw_lines(camera, lines, color)

    #this is slow, out of date and doesn't quite work
    #draw points randomly over faces
    def draw_face_fuzz(self, camera, face, shape, shapes):
        verts = shape.verts[face.get_verts(shape)]
        n_points = 100
        #weights = np.random.uniform(size=[n_points,len(verts)])
        #weights = weights/np.sum(weights,axis=1,keepdims=True)
        #points = np.dot(weights,verts)
        t_vals = np.random.uniform(size=[n_points, 2])
        v0 = shape.verts[shape.edges[face.edges[0]][0]]
        v1 = shape.verts[shape.edges[face.edges[0]][1]]
        v2 = shape.verts[shape.edges[face.edges[2]][0]]
        points = np.vectorize(
            lambda t: line_interpolate(t[0], v0, v1) + line_interpolate(
                t[1], v0, v2),
            signature='(l)->(n)')(t_vals)
        #print(points.shape)
        if camera.clipping:
            clipped = np.full([len(points)], False)
            for clipping_shape in shapes:
                if (clipping_shape is
                        not shape) and (not clipping_shape.transparent):
                    clipped = np.logical_and(
                        clipped,
                        np.vectorize(
                            lambda point: Clipping.point_clipped(
                                point, clipping_shape.boundaries),
                            signature='(n)->()')(points))
            clipped_points = points[np.logical_not(clipped)]
            if len(clipped_points) < 1:
                return
            self.draw_points(camera, clipped_points, face.color)
        else:
            self.draw_points(camera, points, face.color)


class DrawOpenGL2D(Draw):
    def __init__(self, pygame, size, draw_origin, focal=4, screen_scale=60):
        super().__init__(pygame, size, draw_origin, focal)
        self.screen_scale = screen_scale
        self.screen = self.pygame.display.set_mode(
            self.size, pygame.HWSURFACE | pygame.OPENGL | pygame.DOUBLEBUF)
        self.initGL()

    def initGL(self):
        glViewport(0, 0, self.width, self.height)
        glMatrixMode(GL_PROJECTION)
        glLoadIdentity()
        glOrtho(0.0, self.width, 0.0, self.height, 0.0, 1.0)
        glMatrixMode(GL_MODELVIEW)
        glLoadIdentity()
        print('init GL')

    def init_draw(self):
        glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT)  # clear the screen
        #glLoadIdentity()                                   # reset position

    #consider removing duplicate consecutive points
    #since draw_lines_2d converts list of lines to list of points, and connects the dots
    def draw_lines(self, camera, lines, color):
        if len(lines) < 1:
            return

        lines_rel = [line_map(camera.transform, line) for line in lines]
        #clip lines to front of camera (z>z0)
        #remove all lines that are completely clipped (for which clip_line_z0 returns None)
        clipped_lines = remove_None(
            [Clipping.clip_line_z0(line, z0, small_z) for line in lines_rel])
        #don't draw anything if there isn't anything to draw
        if len(clipped_lines) < 1:
            return

        projected_lines = [
            line_map(self.project, line) for line in clipped_lines
        ]
        #clip to viewing sphere
        sphere_clipped_lines = remove_None([
            Clipping.clip_line_sphere(line, r=self.view_radius)
            for line in projected_lines
        ])
        if len(sphere_clipped_lines) < 1:
            return

        try:
            self.draw_lines_2d(sphere_clipped_lines, color)
        except:
            print('problem drawing', sphere_clipped_lines)
            raise

    def draw_points(self, camera, points, color):
        points_rel = camera.transform(points)
        not_clipped = np.vectorize(
            lambda point_rel: not Clipping.point_clipped(
                point_rel, [HyperPlane(np.array([0, 0, 1]), small_z)]),
            signature='(n)->()')(points_rel)
        #only draw if z > 0
        clipped_points = points_rel[not_clipped]
        if len(clipped_points) < 1:
            return
        projected_points = np.vectorize(
            self.project, signature='(n)->(m)')(clipped_points)
        #clip into circle
        in_circle = np.vectorize(
            lambda point: np.dot(point, point) < self.view_radius**2,
            signature='(n)->()')(projected_points)
        projected_points = projected_points[in_circle]
        #point_2d = stuff
        #try:
        self.draw_points_2d(projected_points, color)
        #except:
        #    print('problem drawing',points)
    def draw_points_2d(self, points, color, line_width=2):
        points = (points) * self.screen_scale + self.center
        glColor3f(*color)
        #draw each point as a line with identical start and end points
        glLineWidth(line_width)
        for point in points:
            glBegin(GL_LINES)
            glVertex2f(*point)
            glVertex2f(*(point + np.array([1, 0])))
            glEnd()

    def shift_scale_point(self, point):
        return point * self.screen_scale + self.center

    def draw_lines_2d(self, lines, color, line_width=2):
        glColor3f(*color)
        glLineWidth(line_width)
        glBegin(GL_LINES)
        for line in lines:
            for point in line:
                glVertex2f(*self.shift_scale_point(point))
        glEnd()


class DrawOpenGL3D(Draw):
    def __init__(self, pygame, size, draw_origin, focal=4, stereo=True):
        super().__init__(pygame, size, draw_origin, focal)
        self.screen = self.pygame.display.set_mode(
            self.size, pygame.HWSURFACE | pygame.OPENGL | pygame.DOUBLEBUF)
        self.initGL()
        self.resize(*size)
        self.stereo = stereo
        self.stereo_sep = vec.Vec([5, 0, 0])

        self.view_angles = [30, 30]
        self.stereo_view_angles = [[30, 30], [120, 30]]

    def initGL(self):
        glClearColor(0.0, 0.0, 0.0, 1.0)
        # Set background color to black and opaque
        glClearDepth(1.0)
        # Set background depth to farthest
        glEnable(GL_DEPTH_TEST)
        # Enable depth testing for z-culling
        glDepthFunc(GL_LEQUAL)
        # Set the type of depth-test
        glShadeModel(GL_SMOOTH)
        # Enable smooth shading
        glHint(GL_PERSPECTIVE_CORRECTION_HINT, GL_NICEST)
        # Nice perspective corrections
        self.enable_smoothing()
        #self.init_camera()
        #self.set_camera([0.,10.,0.])
    def draw(self, camera, shapes):
        super().draw(camera, shapes)

        if self.stereo:
            for dorigin, angles in zip([self.stereo_sep, -self.stereo_sep],
                                       self.stereo_view_angles):
                self.draw_bounding_sphere(self.draw_origin + dorigin, angles)
        else:
            self.draw_bounding_sphere(self.draw_origin,
                                      self.stereo_view_angles[0])

    def draw_bounding_sphere(self, draw_origin, draw_angles):
        self.draw_circle_3d(
            self.view_radius, [0, 1],
            GRAY,
            draw_origin=draw_origin,
            draw_angles=draw_angles)
        self.draw_circle_3d(
            self.view_radius, [1, 2],
            GRAY,
            draw_origin=draw_origin,
            draw_angles=draw_angles)
        self.draw_circle_3d(
            self.view_radius, [2, 0],
            GRAY,
            draw_origin=draw_origin,
            draw_angles=draw_angles)


#     def init_camera(self):
#         self.rotation_direction = vector3.Vector3()
#         self.rotation_direction.set(0.0, 0.0, 0.0)
#         self.camera_matrix = matrix44.Matrix44()
#         self.camera_matrix.translate = (0.0,0.0,0.0)

#     def set_camera(self,rot_dir_vec):
#         # Calculate rotation matrix and multiply by camera matrix
#         self.rotation_direction.set(*rot_dir_vec)
#         rotation_matrix = matrix44.Matrix44.xyz_rotation(*self.rotation_direction)
#         self.camera_matrix = rotation_matrix

#         # Calcluate movment and add it to camera matrix translate
# #         heading = Vector3(camera_matrix.forward)
# #         movement = heading * movement_direction.z * movement_speed
# #         self.camera_matrix.translate += movement * time_passed_seconds

#         # Upload the inverse camera matrix to OpenGL
#         glLoadMatrixd(self.camera_matrix.get_inverse().to_opengl())

    def enable_smoothing(self):
        glEnable(GL_LINE_SMOOTH)
        glHint(GL_LINE_SMOOTH_HINT, GL_NICEST)
        glEnable(GL_POINT_SMOOTH)
        glHint(GL_POINT_SMOOTH_HINT, GL_NICEST)
        glEnable(GL_BLEND)
        glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA)

    def resize(self, width, height):  # GLsizei for non-negative integer
        # Compute aspect ratio of the new window
        if (height == 0):
            height = 1
            # To prevent divide by 0
        aspect = width / height

        # Set the viewport to cover the new window
        glViewport(0, 0, width, height)

        # Set the aspect ratio of the clipping volume to match the viewport
        glMatrixMode(GL_PROJECTION)
        # To operate on the Projection matrix
        glLoadIdentity()
        # Reset
        # Enable perspective projection with fovy, aspect, zNear and zFar
        gluPerspective(45.0, aspect, 0.1, 100.0)

    def init_draw(self):
        glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT)  # clear the screen
        glMatrixMode(GL_MODELVIEW)

    def draw_lines(self, camera, lines, color):
        if len(lines) < 1:
            return
        lines_rel = [line_map(camera.transform, line) for line in lines]
        #clip lines to front of camera (z>z0)
        #remove all lines that are completely clipped (for which clip_line_z0 returns None)
        clipped_lines = remove_None(
            [Clipping.clip_line_z0(line, z0, small_z) for line in lines_rel])
        #don't draw anything if there isn't anything to draw
        if len(clipped_lines) < 1:
            return

        projected_lines = [
            line_map(self.project, line) for line in clipped_lines
        ]
        #clip projected lines to unit sphere
        sphere_clipped_lines = remove_None([
            Clipping.clip_line_sphere(line, r=self.view_radius)
            for line in projected_lines
        ])
        if len(sphere_clipped_lines) < 1:
            return

        #remove lines with points outside of cube
        #print(projected_lines.shape)
        # projected_lines = projected_lines[projected_lines[:,:,0] < 1.]
        try:
            if self.stereo:
                self.draw_lines_3d(
                    sphere_clipped_lines,
                    color,
                    draw_origin=self.draw_origin + self.stereo_sep,
                    draw_angles=self.stereo_view_angles[0])
                self.draw_lines_3d(
                    sphere_clipped_lines,
                    color,
                    draw_origin=self.draw_origin - self.stereo_sep,
                    draw_angles=self.stereo_view_angles[1])
            else:
                self.draw_lines_3d(
                    sphere_clipped_lines,
                    color,
                    draw_origin=self.draw_origin,
                    draw_angles=self.view_angles)
        except:
            print('problem drawing', projected_lines)
            raise

    def draw_lines_3d(self,
                      lines,
                      color,
                      line_width=2,
                      draw_origin=None,
                      draw_angles=[30, 30]):
        if draw_origin is None:
            draw_origin = self.draw_origin
        glLoadIdentity()  # reset position
        glTranslatef(*draw_origin)
        #origin of plotting
        glRotatef(draw_angles[1], 1, 0, 0)
        glRotatef(draw_angles[0], 0, 1, 0)

        glColor3f(*color)
        glLineWidth(line_width)
        glBegin(GL_LINES)
        for line in lines:
            for point in line:
                glVertex3f(*point)
        glEnd()

    def draw_circle_3d(self,
                       radius,
                       axes,
                       color,
                       n_points=30,
                       line_width=2,
                       draw_origin=None,
                       draw_angles=[30, 30]):
        if draw_origin is None:
            draw_origin = self.draw_origin
        glLoadIdentity()  # reset position
        glTranslatef(*draw_origin)
        #origin of plotting
        glRotatef(draw_angles[1], 1, 0, 0)
        glRotatef(draw_angles[0], 0, 1, 0)
        glColor3f(*color)
        glLineWidth(line_width)
        glBegin(GL_LINE_LOOP)
        for i in range(n_points):
            x = math.cos(math.pi * 2 * i / n_points) * radius
            y = math.sin(math.pi * 2 * i / n_points) * radius
            p = vec.zero_vec(3)
            p[axes[0]] = x
            p[axes[1]] = y
            glVertex3f(*p)
        glEnd()