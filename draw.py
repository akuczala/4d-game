import Clipping
import vec

import numpy as np
from Geometry import line_map, HyperPlane

from util import remove_None, flatten
import math

import colors

import draw as this
import opengl_draw_3d
import opengl_draw_2d

small_z = 0.001
z0 = 0

def init(d, size, focal=6, view_radius=5, stereo=False):
    this.d = d

    #this.pygame = pygame
    this.focal = focal
    this.view_radius = view_radius
    #scale faces very slightly to avoid drawing edges on top of each other
    #also, for some reason, setting this to one leads to a divide by 0 error
    #when there's transparency
    this.face_scales = [0.95]

    this.bounds_color = colors.GRAY
    if d == 3:
        this.graphics = opengl_draw_2d
        this.draw_origin = vec.zero_vec(d)
        this.graphics.init(size, this.draw_origin)
    if d == 4:
        this.draw_origin = vec.Vec([0.0,0.0,-15.0])
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


def draw(camera, shapes):

    this.graphics.init_draw()

    for shape in shapes:
        shape.update_visibility(camera)
        shape.boundaries = Clipping.calc_boundaries(shape.faces,
                                                    shape.subfaces, camera.pos)

    for shape in shapes:

        for face in shape.faces:
            if face.visible:
                color = face.color

                #calculate scaled lines, for aesthetics
                def scale_point(p, scale):
                    return vec.linterp(face.center, p, scale)

                #this.draw_face_fuzz(camera,face,shape,shapes)
                lines = [shape.get_edgei_line(ei) for ei in face.edgeis]

                scaled_lines = flatten(
                    [[line_map(scale_point, p, scale_face) for p in lines]
                     for scale_face in this.face_scales])
                lines = scaled_lines

                #clip things behind camera first
                lines = remove_None([
                    Clipping.clip_line_plane(line, camera.plane, small_z)
                    for line in lines
                ])
                if len(lines) > 1:
                    #clipping = False doubles the framerate
                    if this.clipping:
                        clipped_lines = Clipping.clip_lines(
                            lines, shape, shapes)
                        #draw clipped line
                        draw_lines(camera, clipped_lines, color)
                    else:  #noclip
                        draw_lines(camera, lines, color)
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
                                      this.stereo_view_angles[0],
                                      this.bounds_color)


#transforms lines to camera space and clips lines behind the camera,
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
    #clip to viewing sphere
    sphere_clipped_lines = remove_None([
        Clipping.clip_line_sphere(line, r=this.view_radius)
        for line in projected_lines
    ])
    if len(sphere_clipped_lines) < 1:
        return []

    return sphere_clipped_lines


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
        this.draw_points(camera, clipped_points, face.color)
    else:
        this.draw_points(camera, points, face.color)


#consider removing duplicate consecutive points
#since draw_lines_2d converts list of lines to list of points, and connects the dots
def draw_lines(camera, lines, color):
    sphere_clipped_lines = clip_project_lines(camera, lines, color)
    if len(sphere_clipped_lines) < 1:
        return
    try:
        if this.d == 3:
            this.graphics.draw_lines_2d(sphere_clipped_lines, color)
        if this.d == 4:
            if this.stereo:
                this.graphics.draw_lines_3d(
                    sphere_clipped_lines,
                    color,
                    draw_origin=this.draw_origin + this.stereo_sep,
                    draw_angles=this.stereo_view_angles[0])
                this.graphics.draw_lines_3d(
                    sphere_clipped_lines,
                    color,
                    draw_origin=this.draw_origin - this.stereo_sep,
                    draw_angles=this.stereo_view_angles[1])
            else:
                this.graphics.draw_lines_3d(
                    sphere_clipped_lines,
                    color,
                    draw_origin=this.draw_origin,
                    draw_angles=this.view_angles)
    except:
        print('problem drawing', sphere_clipped_lines)
        raise


#out of date
def draw_points(this, camera, points, color):
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
        this.project, signature='(n)->(m)')(clipped_points)
    #clip into circle
    in_circle = np.vectorize(
        lambda point: np.dot(point, point) < this.view_radius**2,
        signature='(n)->()')(projected_points)
    projected_points = projected_points[in_circle]
    #point_2d = stuff
    #try:
    this.draw_points_2d(projected_points, color)
    #except:
    #    print('problem drawing',points)


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