import OpenGL.GL as gl
from OpenGL.GL import glBegin, glEnd, glLineWidth, glVertex3f
import opengl_draw_3d as this
from OpenGL.GL import glColor3f, GL_LINES, GL_LINE_LOOP
import OpenGL.GLU as glu
import pygame
import vec
import math


def init(size, draw_origin=vec.Vec([0.0, 0.0, -15.0]), focal=4, stereo=True):
    this.size = size
    this.size = size
    this.width, this.height = size
    this.screen = pygame.display.set_mode(
        this.size, pygame.HWSURFACE | pygame.OPENGL | pygame.DOUBLEBUF)
    initGL()
    resize(*size)


def resize(width, height):  # GLsizei for non-negative integer
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


def initGL():
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
    enable_smoothing()
    #self.init_camera()
    #self.set_camera([0.,10.,0.])


def enable_smoothing():
    gl.glEnable(gl.GL_LINE_SMOOTH)
    gl.glHint(gl.GL_LINE_SMOOTH_HINT, gl.GL_NICEST)
    gl.glEnable(gl.GL_POINT_SMOOTH)
    gl.glHint(gl.GL_POINT_SMOOTH_HINT, gl.GL_NICEST)
    gl.glEnable(gl.GL_BLEND)
    gl.glBlendFunc(gl.GL_SRC_ALPHA, gl.GL_ONE_MINUS_SRC_ALPHA)


def init_draw():
    gl.glClear(gl.GL_COLOR_BUFFER_BIT
               | gl.GL_DEPTH_BUFFER_BIT)  # clear the screen
    gl.glMatrixMode(gl.GL_MODELVIEW)


def draw_lines_3d(lines, color, draw_origin, draw_angles, line_width=2):
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


def draw_sphere(radius, draw_origin, draw_angles, color):
    draw_circle_3d(
        radius, [0, 1],
        color,
        draw_origin=draw_origin,
        draw_angles=draw_angles)
    draw_circle_3d(
        radius, [1, 2],
        color,
        draw_origin=draw_origin,
        draw_angles=draw_angles)
    draw_circle_3d(
        radius, [2, 0],
        color,
        draw_origin=draw_origin,
        draw_angles=draw_angles)

def draw_cylinder(radius, height, draw_origin, draw_angles, color,axis=1):
    circle_axes = [0,1,2]
    circle_axes.pop(axis)
    height_vec = vec.one_hot(3,axis)
    for h in [0,height/2,-height/2]:
        draw_circle_3d(
            radius, circle_axes,
            color,
            draw_origin=draw_origin + h*height_vec,
            draw_angles=draw_angles)

def draw_circle_3d(radius,
                   axes,
                   color,
                   draw_origin,
                   draw_angles,
                   n_points=30,
                   line_width=2):
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


def draw_points_3d(points, color, draw_origin, draw_angles, line_width=2):
    gl.glLoadIdentity()  # reset position
    gl.glTranslatef(*draw_origin)
    #origin of plotting
    gl.glRotatef(draw_angles[1], 1, 0, 0)
    gl.glRotatef(draw_angles[0], 0, 1, 0)

    glColor3f(*color)
    #draw each point as a line with (almost) identical start and end points
    glLineWidth(line_width)
    for point in points:
        glBegin(GL_LINES)
        glVertex3f(*point)
        glVertex3f(*(point + vec.Vec([0.01, 0, 0])))
        glEnd()