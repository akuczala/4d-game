import OpenGL.GL as gl
from OpenGL.GL import glBegin, glEnd, glLineWidth, glVertex2f
import opengl_draw_2d as this
from OpenGL.GL import glColor3f, GL_LINES, GL_LINE_LOOP
import pygame
import vec
import math

width = 800
height = 600

def init(size, draw_origin, screen_scale=15):
    this.size = size
    this.width, this.height = size
    this.center = (this.width // 2, this.height // 2)
    this.draw_origin = draw_origin
    this.screen_scale = screen_scale
    this.screen = pygame.display.set_mode(
        size, pygame.HWSURFACE | pygame.OPENGL | pygame.DOUBLEBUF
        | pygame.RESIZABLE)

    initGL()

def initGL():
    print('init GL')
    gl.glViewport(0, 0, this.width, this.height)
    gl.glMatrixMode(gl.GL_PROJECTION)
    gl.glLoadIdentity()
    gl.glOrtho(0.0, this.width, 0.0, this.height, 0.0, 1.0)
    gl.glMatrixMode(gl.GL_MODELVIEW)
    gl.glLoadIdentity()
    enable_smoothing()

def init_draw():
    gl.glClear(gl.GL_COLOR_BUFFER_BIT | gl.GL_DEPTH_BUFFER_BIT)  # clear the screen
    #glLoadIdentity()                                   # reset position


def enable_smoothing():
    gl.glEnable(gl.GL_LINE_SMOOTH)
    gl.glHint(gl.GL_LINE_SMOOTH_HINT, gl.GL_NICEST)
    gl.glEnable(gl.GL_POINT_SMOOTH)
    gl.glHint(gl.GL_POINT_SMOOTH_HINT, gl.GL_NICEST)
    gl.glEnable(gl.GL_BLEND)
    gl.glBlendFunc(gl.GL_SRC_ALPHA, gl.GL_ONE_MINUS_SRC_ALPHA)

def shift_scale_point(point):
    return point * this.height / this.screen_scale + this.center

def draw_lines_2d(lines, color, line_width=2):

    glColor3f(*color)
    glLineWidth(line_width)
    glBegin(GL_LINES)
    for line in lines:
        for point in line:
            glVertex2f(*shift_scale_point(point))
    glEnd()

def draw_circle_2d(radius,
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
        glVertex2f(*shift_scale_point(p))
    glEnd()

def draw_points_2d(points, color, line_width=2):
    glColor3f(*color)
    #draw each point as a line with (almost) identical start and end points
    glLineWidth(line_width)
    for point in points:
        glBegin(GL_LINES)
        glVertex2f(*shift_scale_point(point))
        glVertex2f(*(shift_scale_point(point) + vec.Vec([1, 0])))
        glEnd()