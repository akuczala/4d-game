import Clipping
import vec
import itertools
from vec import Vec
import numpy as np
from Geometry import line_map
from Geometry import Line
from util import remove_None, flatten
import math

import colors

import draw as this
import opengl_draw_3d
import opengl_draw_2d

small_z = 0.001
z0 = 0


def init(d, size, focal=6, view_radius=5, view_height = 5, stereo=False, n_fuzz_points=10):
    this.d = d

    #this.pygame = pygame
    this.focal = focal
    this.view_radius = view_radius
    this.view_height = view_height
    this.view_boundary = 'sphere'
    #scale faces very slightly to avoid drawing edges on top of each other
    #also, for some reason, setting this to one leads to a divide by 0 error
    #when there's transparency
    this.face_scales = [0.95]

    this.bounds_color = colors.GRAY
    this.show_fuzz = False
    this.random_fuzz = np.random.uniform(size=[n_fuzz_points,
                                               d - 1])  #for face fuzz
    this.width = size[0]
    this.height = size[1]
    this.center = (this.width // 2, this.height // 2)  #needed for mouse input

    if d == 3:
        this.graphics = opengl_draw_2d
        this.draw_origin = vec.zero_vec(d)
        this.graphics.init(size, this.draw_origin)
    if d == 4:
        this.draw_origin = vec.Vec([0.0, 0.0, -15.0])
        this.graphics = opengl_draw_3d
        this.graphics.init(size, this.draw_origin)
        this.stereo = stereo
        this.stereo_sep = vec.Vec([5, 0, 0])

        this.view_angles = [30, 30]
        this.stereo_view_angles = [[30, 30], [120, 30]]


#project d-dimensional vector to d-1 dimensional vector
def project(v):
    if math.isclose(v[-1], z0):
        z = z0 + small_z
    else:
        z = v[-1]
    return this.focal * v[:-1] / (z)

def draw_wireframe(camera,shape,color):
    #init?
    lines = [shape.get_edge_line(edge) for edge in shape.edges]
    lines = camera_clip_lines(lines,camera)
    if len(lines) < 1:
        return
    draw_lines(camera, lines, color)

def draw_normals(camera,shape,color):
    lines = [Line(face.center,face.center + face.normal) for face in shape.faces]
    lines = camera_clip_lines(lines,camera)
    if len(lines) < 1:
        return
    draw_lines(camera, lines, color)
def draw(camera, shapes):
    #initialize frame
    this.graphics.init_draw()

    for shape in shapes:
        shape.update_visibility(camera)
        #calculate boundaries for clipping
        if this.clipping:
            shape.boundaries = Clipping.calc_boundaries(
                shape.faces, shape.subfaces, camera.pos)
    #draw shapes
    for shape in shapes:

        for face in shape.faces:
            if face.visible:
                if this.show_fuzz:
                    draw_face_fuzz(camera, face, shape, shapes)

                draw_face_lines(camera, face, shape, shapes)

    #draw boundary of (d-1) screen
    if this.view_boundary == 'sphere':
        draw_spherical_boundary()
    if this.view_boundary == 'cylinder':
        draw_cylindrical_boundary()


def draw_spherical_boundary():
    if this.d == 3:
        this.graphics.draw_circle_2d(this.view_radius, this.bounds_color)
    if this.d == 4:
        if this.stereo:
            for dorigin, angles in zip([this.stereo_sep, -this.stereo_sep],
                                       this.stereo_view_angles):
                this.graphics.draw_sphere(this.view_radius,
                                          this.draw_origin + dorigin, angles,
                                          this.bounds_color)
        else:
            this.graphics.draw_sphere(this.view_radius, this.draw_origin,
                                      this.view_angles, this.bounds_color)


def draw_cylindrical_boundary():
    if this.d == 3:
        this.graphics.draw_rectangle(this.view_radius*2, this.view_height, this.bounds_color)
    if this.d == 4:
        if this.stereo:
            for dorigin, angles in zip([this.stereo_sep, -this.stereo_sep],
                                       this.stereo_view_angles):
                this.graphics.draw_cylinder(
                    this.view_radius, this.view_height,
                    this.draw_origin + dorigin, angles, this.bounds_color,axis=1)
        else:
            this.graphics.draw_cylinder(
                this.view_radius, this.view_height, this.draw_origin,
                this.view_angles, this.bounds_color,axis=1)

#lines should be cached somewhere
#works only for d=3
def draw_floor(scale,h,color,camera,shapes):
    n = 10
    r = n*scale/2
    lines = [Line(Vec([scale*i-r,h,scale*j-r]),Vec([scale*i-r,h,scale*(j+1)-r])) for i,j in itertools.product(range(n),range(n))]
    lines = lines + [Line(Vec([scale*i-r,h,scale*j-r]),Vec([scale*(i+1)-r,h,scale*j-r])) for i,j in itertools.product(range(n),range(n))]

    lines = camera_clip_lines(lines,camera)

    if len(lines) > 1:
        #clipping = False doubles the framerate
        if this.clipping:
            clipped_lines = Clipping.clip_lines(lines, None, shapes)
            #draw clipped line
            draw_lines(camera, clipped_lines, color)
        else:  #noclip
            draw_lines(camera, lines, color)

def draw_face_lines(camera, face, shape, shapes):
    color = face.color

    #calculate scaled lines, for aesthetics
    def scale_point(p, scale):
        return vec.linterp(face.center, p, scale)

    lines = [shape.get_edgei_line(ei) for ei in face.edgeis]
    #lines = []

    scaled_lines = flatten(
        [[line_map(scale_point, p, scale_face) for p in lines]
         for scale_face in this.face_scales])
    lines = scaled_lines

    lines = camera_clip_lines(lines,camera)

    if len(lines) > 1:
        #clipping = False doubles the framerate
        if this.clipping:
            clipped_lines = Clipping.clip_lines(lines, shape, shapes)
            #draw clipped line
            draw_lines(camera, clipped_lines, color)
        else:  #noclip
            draw_lines(camera, lines, color)

#clip things behind camera
def camera_clip_lines(lines,camera):
    return remove_None([
        Clipping.clip_line_plane(line, camera.plane, small_z) for line in lines
    ])
#transforms lines to camera space
#then projects the lines down to d-1 and does viewport clipping
def clip_project_lines(camera, lines, color):
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

    projected_lines = [line_map(project, line) for line in lines_rel]
    #clip to viewing region
    if this.view_boundary == 'sphere':
        view_clipped_lines = remove_None([
            Clipping.clip_line_sphere(line, r=this.view_radius)
            for line in projected_lines
        ])
    if this.view_boundary == 'cylinder':
        view_clipped_lines = remove_None([
            Clipping.clip_line_cylinder(
                line, r=this.view_radius, h=this.view_height, axis=1)
            for line in projected_lines
        ])
    if this.view_boundary == 'none':
        view_clipped_lines = projected_lines
    if len(view_clipped_lines) < 1:
        return []

    return view_clipped_lines


#out of date
def draw_frame_lines(camera):
    d = len(camera.pos)
    frame_origin = camera.frame[-1] * 0.1
    frame_origin += camera.pos
    frame_lines = np.stack((np.zeros([d, d]), camera.frame)).transpose(1, 0, 2)
    frame_lines = frame_lines * 0.5 + frame_origin
    for frame_line, color in zip(frame_lines, [
            colors.PURPLE, colors.MAGENTA, colors.ORANGE, colors.CYAN
    ][:d]):
        this.draw_lines(camera, [frame_line], color)


#this is slow, out of date and doesn't quite work
#draw points randomly over faces
def draw_face_fuzz(camera, face, shape, shapes):
    #weights = np.random.uniform(size=[n_points,len(verts)])
    #weights = weights/np.sum(weights,axis=1,keepdims=True)
    #points = np.dot(weights,verts)

    verts = [shape.verts[i] for i in face.get_verts(shape)]
    #points = [vec.linterp(verts[0], verts[1], t[0]) + vec.linterp(verts[0], verts[2], t[1]) for t in t_vals]
    if this.d == 3:
        points = [
            vec.linterp(
                vec.linterp(verts[0], verts[1], t[0]),
                vec.linterp(verts[2], verts[3], t[0]), t[1])
            for t in this.random_fuzz
        ]
    if this.d == 4:
        points = [
            vec.linterp(
                vec.linterp(
                    vec.linterp(verts[0], verts[1], t[0]),
                    vec.linterp(verts[2], verts[3], t[0]), t[1]),
                vec.linterp(
                    vec.linterp(verts[4], verts[5], t[0]),
                    vec.linterp(verts[6], verts[7], t[0]), t[1]), t[2])
            for t in this.random_fuzz
        ]
    #print(points.shape)
    if this.clipping:
        clipped = [False for i in range(len(points))]
        for clipping_shape in shapes:
            if (clipping_shape is
                    not shape) and (not clipping_shape.transparent):
                new_clipped = [
                    Clipping.point_clipped(point, clipping_shape.boundaries)
                    for point in points
                ]
                clipped = [
                    clip1 or clip2
                    for clip1, clip2 in zip(clipped, new_clipped)
                ]
        clipped_points = [
            point for point, clip in zip(points, clipped) if (not clip)
        ]
        if len(clipped_points) < 1:
            return
        draw_points(camera, clipped_points, face.color)
    else:
        draw_points(camera, points, face.color)


#consider removing duplicate consecutive points
#since draw_lines_2d converts list of lines to list of points, and connects the dots
def draw_lines(camera, lines, color):
    view_clipped_lines = clip_project_lines(camera, lines, color)
    if len(view_clipped_lines) < 1:
        return
    try:
        if this.d == 3:
            this.graphics.draw_lines_2d(view_clipped_lines, color)
        if this.d == 4:
            if this.stereo:
                this.graphics.draw_lines_3d(
                    view_clipped_lines,
                    color,
                    draw_origin=this.draw_origin + this.stereo_sep,
                    draw_angles=this.stereo_view_angles[0])
                this.graphics.draw_lines_3d(
                    view_clipped_lines,
                    color,
                    draw_origin=this.draw_origin - this.stereo_sep,
                    draw_angles=this.stereo_view_angles[1])
            else:
                this.graphics.draw_lines_3d(
                    view_clipped_lines,
                    color,
                    draw_origin=this.draw_origin,
                    draw_angles=this.view_angles)
    except:
        print('problem drawing', view_clipped_lines)
        raise


def draw_points(camera, points, color):

    clipped_points = [
        point for point in points
        if (not Clipping.point_clipped(point, [camera.plane], small_z))
    ]

    #clipped_points = points #DEBUG

    if len(clipped_points) < 1:
        return

    points_rel = camera.transform(clipped_points)

    projected_points = [project(point) for point in points_rel]

    #need to implement clipping into cylinder
    #clip into sphere
    projected_points = [
        point for point in projected_points
        if np.dot(point, point) < this.view_radius**2
    ]

    try:
        if this.d == 3:
            this.graphics.draw_points_2d(projected_points, color)

        if this.d == 4:
            if this.stereo:
                for dorigin, angles in zip([this.stereo_sep, -this.stereo_sep],
                                           this.stereo_view_angles):
                    this.graphics.draw_points_3d(projected_points, color,
                                                 this.draw_origin + dorigin,
                                                 angles)
            else:
                this.graphics.draw_points_3d(projected_points, color,
                                             this.draw_origin,
                                             this.view_angles)
    except:
        print('problem drawing', points)
        raise


#     def init_camera(this):
#         this.rotation_direction = vector3.Vector3()
#         this.rotation_direction.set(0.0, 0.0, 0.0)
#         this.camera_matrix = matrix44.Matrix44()
#         this.camera_matrix.translate = (0.0,0.0,0.0)

#     def set_camera(this,rot_dir_vec):
#         # Calculate rotation matrix and multiply by camera matrix
#         this.rotation_direction.set(*rot_dir_vec)
#         rotation_matrix = matrix44.Matrix44.xyz_rotation(*this.rotation_direction)
#         this.camera_matrix = rotation_matrix

#         # Calcluate movment and add it to camera matrix translate
# #         heading = Vector3(camera_matrix.forward)
# #         movement = heading * movement_direction.z * movement_speed
# #         this.camera_matrix.translate += movement * time_passed_seconds

#         # Upload the inverse camera matrix to OpenGL
#         glLoadMatrixd(this.camera_matrix.get_inverse().to_opengl())