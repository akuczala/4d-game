import numpy as np
import numpy.linalg as lin

#let vectors be numpy arrays
Vec = np.array
Matrix = np.array

#np.array gives most of the features we want for our vector class


def dim(x):
    return len(x)


def dot(x, y):
    return np.dot(x, y)


def norm(x):
    return lin.norm(x)


def linf_norm(x):
    return lin.norm(x, ord=np.inf)


def barycenter(vecs):
    return np.mean(vecs, axis=0)


def isclose(x, y):
    return np.isclose(x, y)


def zero_vec(d):
    return np.zeros((d))


def zero_mat(d):
    return np.zeros((d, d))


def ones_vec(d):
    return np.ones((d))


def eye(n):
    return np.eye(n)


def rotmat(t):
    return np.array([[np.cos(t), np.sin(t)], [-np.sin(t), np.cos(t)]])


#finds rotation matrix between two (normalized) vectors (rotates v1 to v2)
def rotation_matrix(v1, v2, th=None):
    u = v1 / norm(v1)
    v = v2 / norm(v2)
    if th is None:
        costh = np.dot(u, v)
        sinth = np.sqrt(1 - costh**2)
    else:
        costh = np.cos(th)
        sinth = np.sin(th)
    #sinth = np.sin(np.arccos(costh))
    R = Matrix([[costh, -sinth], [sinth, costh]])
    w = (v - dot(u, v) * u)
    w = w / norm(w)
    uw_mat = np.array([u, w])
    return np.eye(len(u)) - np.outer(u, u) - np.outer(w, w) + np.dot(
        uw_mat.T, np.dot(R, uw_mat))


def rotation_matrix_aligned(dir1, dir2, th):
    costh = np.cos(th)
    sinth = np.sin(th)
    if dir1 == 0 and dir2 == 2:
        return Matrix([[costh, 0, -sinth], [0, 1, 0], [sinth, 0, costh]])
    if dir1 == 1 and dir2 == 2:
        return Matrix([[1, 0, 0], [0, costh, -sinth], [0, sinth, costh]])


#linear interpolation between vectors p1 at x=0 and p2 at x=1
#0 ⩽ x ⩽ 1
def linterp(v1, v2, x):
    return v1 * (1 - x) + v2 * x

#return vector with index removed (used for cylinder clipping)
def drop_index(v,index):
    return Vec([v[i] for i in range(len(v)) if i != index])
def insert_index(v,index,vi):
    return np.insert(v,index,vi)
def one_hot(d,index):
    return np.eye(d)[index]