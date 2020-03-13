from OpenGL.GL import *
from OpenGL.GLU import *

def resize(width, height):
    
    glViewport(0, 0, width, height)
    glMatrixMode(GL_PROJECTION)
    glLoadIdentity()
    gluPerspective(60.0, float(width)/height, .1, 1000.)
    glMatrixMode(GL_MODELVIEW)
    glLoadIdentity()

def refresh(width, height):
    glViewport(0, 0, width, height)
    glMatrixMode(GL_PROJECTION)
    glLoadIdentity()
    glOrtho(0.0, width, 0.0, height, 0.0, 1.0)
    glMatrixMode (GL_MODELVIEW)
    glLoadIdentity()
def draw_rectangle(x,y,width,height):
    glBegin(GL_LINES)                                  # start drawing a rectangle
    glVertex2f(x, y)                                   # bottom left point
    glVertex2f(x + width, y)                           # bottom right point
    glVertex2f(x + width, y + height)                  # top right point
    glVertex2f(x, y + height)                          # top left point
    glEnd()  
    return
def display(SCREEN_SIZE):
	glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT) # clear the screen
	glLoadIdentity()  # reset position
	#resize(*SCREEN_SIZE)
	glColor3f(0.0,1.,1.0) 
	glLineWidth(2.5)
	#draw_line(int(400 + 100*np.cos(t)),int(400+100*np.sin(t)),400,400)
	draw_rectangle(100,200,50,100)