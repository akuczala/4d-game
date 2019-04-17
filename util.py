from functools import reduce
from operator import iconcat


# return list of unique elements (does not preserve order)
def unique(some_list):
    return list(set(some_list))

    
# flattens list of lists
# fastest flatten according to
# https://stackoverflow.com/a/45323085
def flatten(list_of_lists):
    return reduce(iconcat, list_of_lists, [])


# usually list comprehension will do this
def remove_None(some_list):
    return list(filter(None.__ne__, some_list))