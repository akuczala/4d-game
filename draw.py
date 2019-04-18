import Clipping
import vec

import numpy as np
from Geometry import line_map, HyperPlane

from util import remove_None, flatten
import math
import pygame
import colors

import OpenGL.GL as gl
import OpenGL.GLU as glu
from OpenGL.GL import glBegin, glEnd, glLineWidth, glVertex2f, glVertex3f
from OpenGL.GL import glColor3f, GL_LINES, GL_LINE_LOOP

small_z = 0.001
z0 = 0

#focal = 4.


def to_screen(v2, scale, center):
    return v2 * scale + center


#maybe keep this a class and make __init__ a normal function?
#can we still do inheritance / should we?
#or, should we turn this into a module?
class Draw:
    def __init__(self, size, draw_origin, focal, view_radius=5):
        self.width, self.height = self.size = size
        self.center = (self.width // 2, self.height // 2)
        #self.pygame = pygame
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

                    #clip things behind camera first
                    lines = remove_None([
                        Clipping.clip_line_plane(line, camera.plane, small_z)
                        for line in lines
                    ])
                    if len(lines) > 1:
                        #clipping = False doubles the framerate
                        if self.clipping:
                            clipped_lines = Clipping.clip_lines(
                                lines, shape, shapes)
                            #draw clipped line
                            self.draw_lines(camera, clipped_lines, color)
                        else:  #noclip
                            self.draw_lines(camera, lines, color)

    #transforms lines to camera space and clips lines behind the camera,
    #then projects the lines down to d-1 and does viewport clipping
    def clip_project_lines(self, camera, lines, color):
        if len(lines) < 1:
            return []

        lines_rel = [line_map(camera.transform, line) for line in lines]

        #clip lines to front of camera (z>z0) (that's the old way to do it!)
        #though it is unclear which way is faster
        # clipped_lines = remove_None(
        #     [Clipping.clip_line_z0(line, z0, small_z) for line in lines_rel])
        # #don't draw anything if there isn't anything to draw
        # if len(clipped_lines) < 1:
        #     return []

        projected_lines = [line_map(self.project, line) for line in lines_rel]
        #clip to viewing sphere
        sphere_clipped_lines = remove_None([
            Clipping.clip_line_sphere(line, r=self.view_radius)
            for line in projected_lines
        ])
        if len(sphere_clipped_lines) < 1:
            return []

        return sphere_clipped_lines

    #out of date
    def draw_frame_lines(self, camera):
        d = len(camera.pos)
        frame_origin = camera.frame[-1] * 0.1
        frame_origin += camera.pos
        frame_lines = np.stack((np.zeros([d, d]), camera.frame)).transpose(
            1, 0, 2)
        frame_lines = frame_lines * 0.5 + frame_origin
        for frame_line, color in zip(frame_lines, [
                colors.PURPLE, colors.MAGENTA, colors.ORANGE, colors.CYAN
        ][:d]):
            self.draw_lines(camera, [frame_line], color)

    #this is slow, out of date and doesn't quite work
    #draw points randomly over faces
    def draw_face_fuzz(self, camera, face, shape, shapes):
        n_points = 100
        #weights = np.random.uniform(size=[n_points,len(verts)])
        #weights = weights/np.sum(weights,axis=1,keepdims=True)
        #points = np.dot(weights,verts)
        t_vals = np.random.uniform(size=[n_points, 2])
        v0 = shape.verts[shape.edges[face.edges[0]][0]]
        v1 = shape.verts[shape.edges[face.edges[0]][1]]
        v2 = shape.verts[shape.edges[face.edges[2]][0]]
        points = np.vectorize(
            lambda t: vec.linterp(v0, v1, t[0]) + vec.linterp(v0, v2, t[1]),
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

    def enable_smoothing(self):
        gl.glEnable(gl.GL_LINE_SMOOTH)
        gl.glHint(gl.GL_LINE_SMOOTH_HINT, gl.GL_NICEST)
        gl.glEnable(gl.GL_POINT_SMOOTH)
        gl.glHint(gl.GL_POINT_SMOOTH_HINT, gl.GL_NICEST)
        gl.glEnable(gl.GL_BLEND)
        gl.glBlendFunc(gl.GL_SRC_ALPHA, gl.GL_ONE_MINUS_SRC_ALPHA)


class DrawOpenGL2D(Draw):
    def __init__(self, size, draw_origin, focal=6, screen_scale=15):
        super().__init__(size, draw_origin, focal)
        self.screen_scale = screen_scale
        self.screen = pygame.display.set_mode(
            self.size, pygame.HWSURFACE | pygame.OPENGL | pygame.DOUBLEBUF
            | pygame.RESIZABLE)

        self.initGL()

    def initGL(self):
        print('init GL')
        gl.glViewport(0, 0, self.width, self.height)
        gl.glMatrixMode(gl.GL_PROJECTION)
        gl.glLoadIdentity()
        gl.glOrtho(0.0, self.width, 0.0, self.height, 0.0, 1.0)
        gl.glMatrixMode(gl.GL_MODELVIEW)
        gl.glLoadIdentity()
        self.enable_smoothing()

    def init_draw(self):
        gl.glClear(gl.GL_COLOR_BUFFER_BIT
                   | gl.GL_DEPTH_BUFFER_BIT)  # clear the screen
        #glLoadIdentity()                                   # reset position

    def draw(self, camera, shapes):
        super().draw(camera, shapes)
        self.draw_circle_2d(self.view_radius, colors.GRAY)

    #merge more with 3D version
    #consider removing duplicate consecutive points
    #since draw_lines_2d converts list of lines to list of points, and connects the dots
    def draw_lines(self, camera, lines, color):
        sphere_clipped_lines = self.clip_project_lines(camera, lines, color)
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
        return point * self.height / self.screen_scale + self.center

    def draw_lines_2d(self, lines, color, line_width=2):

        glColor3f(*color)
        glLineWidth(line_width)
        glBegin(GL_LINES)
        for line in lines:
            for point in line:
                glVertex2f(*self.shift_scale_point(point))
        glEnd()

    def draw_circle_2d(self,
                       radius,
                       color,
                       n_points=60,
                       line_width=2,
                       draw_origin=vec.zero_vec(2)):
        glColor3f(*color)
        glLineWidth(line_width)
        glBegin(GL_LINE_LOOP)
        for i in range(n_points):
            x = math.cos(math.pi * 2 * i / n_points) * radius
            y = math.sin(math.pi * 2 * i / n_points) * radius
            p = vec.Vec([x, y]) + draw_origin
            glVertex2f(*self.shift_scale_point(p))
        glEnd()


class DrawOpenGL3D(Draw):
    def __init__(self, size, draw_origin, focal=4, stereo=True):
        super().__init__(size, draw_origin, focal)
        self.screen = pygame.display.set_mode(
            self.size, pygame.HWSURFACE | pygame.OPENGL | pygame.DOUBLEBUF)
        self.initGL()
        self.resize(*size)
        self.stereo = stereo
        self.stereo_sep = vec.Vec([5, 0, 0])

        self.view_angles = [30, 30]
        self.stereo_view_angles = [[30, 30], [120, 30]]

    def initGL(self):
        print('init GL')
        gl.glClearColor(0.0, 0.0, 0.0, 1.0)
        # Set background color to black and opaque
        gl.glClearDepth(1.0)
        # Set background depth to farthest
        gl.glEnable(gl.GL_DEPTH_TEST)
        # Enable depth testing for z-culling
        gl.glDepthFunc(gl.GL_LEQUAL)
        # Set the type of depth-test
        gl.glShadeModel(gl.GL_SMOOTH)
        # Enable smooth shading
        gl.glHint(gl.GL_PERSPECTIVE_CORRECTION_HINT, gl.GL_NICEST)
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
            colors.GRAY,
            draw_origin=draw_origin,
            draw_angles=draw_angles)
        self.draw_circle_3d(
            self.view_radius, [1, 2],
            colors.GRAY,
            draw_origin=draw_origin,
            draw_angles=draw_angles)
        self.draw_circle_3d(
            self.view_radius, [2, 0],
            colors.GRAY,
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

#NOT USED / probably from sample code

    def resize(self, width, height):  # GLsizei for non-negative integer
        # Compute aspect ratio of the new window
        if (height == 0):
            height = 1
            # To prevent divide by 0
        aspect = width / height

        # Set the viewport to cover the new window
        gl.glViewport(0, 0, width, height)

        # Set the aspect ratio of the clipping volume to match the viewport
        gl.glMatrixMode(gl.GL_PROJECTION)
        # To operate on the Projection matrix
        gl.glLoadIdentity()
        # Reset
        # Enable perspective projection with fovy, aspect, zNear and zFar
        glu.gluPerspective(45.0, aspect, 0.1, 100.0)

    def init_draw(self):
        gl.glClear(gl.GL_COLOR_BUFFER_BIT
                   | gl.GL_DEPTH_BUFFER_BIT)  # clear the screen
        gl.glMatrixMode(gl.GL_MODELVIEW)

    def draw_lines(self, camera, lines, color):
        sphere_clipped_lines = self.clip_project_lines(camera, lines, color)
        if len(sphere_clipped_lines) < 1:
            return
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
            print('problem drawing', sphere_clipped_lines)
            raise

    def draw_lines_3d(self,
                      lines,
                      color,
                      line_width=2,
                      draw_origin=None,
                      draw_angles=[30, 30]):
        if draw_origin is None:
            draw_origin = self.draw_origin
        gl.glLoadIdentity()  # reset position
        gl.glTranslatef(*draw_origin)
        #origin of plotting
        gl.glRotatef(draw_angles[1], 1, 0, 0)
        gl.glRotatef(draw_angles[0], 0, 1, 0)

        gl.glColor3f(*color)
        gl.glLineWidth(line_width)
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
        gl.glLoadIdentity()  # reset position
        gl.glTranslatef(*draw_origin)
        #origin of plotting
        gl.glRotatef(draw_angles[1], 1, 0, 0)
        gl.glRotatef(draw_angles[0], 0, 1, 0)
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