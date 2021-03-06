import pygame
import draw
import inputs as this

trans_keymapping = {
    3: {
        pygame.K_l: (0, 1),
        pygame.K_j: (0, -1),
        pygame.K_i: (1, 1),
        pygame.K_k: (1, -1)
    },
    4: {
        pygame.K_a: (2, -1),
        pygame.K_d: (2, 1),
        pygame.K_l: (0, 1),
        pygame.K_j: (0, -1),
        pygame.K_i: (1, 1),
        pygame.K_k: (1, -1)
    }
}
rot_keymapping = {
    3: {
        pygame.K_a: (0, 1, -1),
        pygame.K_d: (0, 1, 1),
        pygame.K_l: (0, 2, 1),
        pygame.K_j: (0, 2, -1),
        pygame.K_i: (1, 2, 1),
        pygame.K_k: (1, 2, -1)
    },
    4: {
        pygame.K_o: (0, 2, 1),
        pygame.K_u: (0, 2, -1),
        pygame.K_a: (2, 3, -1),
        pygame.K_d: (2, 3, 1),
        pygame.K_l: (0, 3, 1),
        pygame.K_j: (0, 3, -1),
        pygame.K_i: (1, 3, 1),
        pygame.K_k: (1, 3, -1)
    }
}

enable_mouse = False

def input_update(camera, shapes):
    update = False
    quit = False
    keys = pygame.key.get_pressed()
    events = pygame.event.get()

    for event in events:
        #check for events that signal quit (such as closing the window)
        if event.type == pygame.QUIT:
            quit = True

        #actions to take on key up
        if event.type == pygame.KEYUP:
            #toggle transparency
            if event.key == pygame.K_t:
                for shape in shapes:
                    shape.transparent = not shape.transparent
                    update = True
                print('transparency', shapes[0].transparent)
            #toggle clipping
            if event.key == pygame.K_c:
                draw.clipping = not draw.clipping
                print('clipping', draw.clipping)
            #toggle fuzz
            if event.key == pygame.K_f:
                draw.show_fuzz = not draw.show_fuzz
                print('show fuzz', draw.show_fuzz)
            #toggle mouse
            if event.key == pygame.K_m:
                this.enable_mouse = not this.enable_mouse
            if event.key == pygame.K_r:
                camera.reset_frame()
            # #window resize
            # if event.key == pygame.VIDEORESIZE:
            #     draw.width = event.w
            #     draw.height = event.h
            #     #need to reinitialize pygame surface
            #end game
            if event.key == pygame.K_ESCAPE:
                quit = True

    #keys that cause continuous action (e.g. movement)

    #adjust view radius, height
    if keys[pygame.K_PERIOD]:
        draw.view_radius = max(draw.view_radius + 0.1,0)
        draw.view_height = max(draw.view_height + 0.1,0)
        update = True
    if keys[pygame.K_COMMA]:
        draw.view_radius = max(draw.view_radius - 0.1,0)
        draw.view_height = max(draw.view_height - 0.1,0)
        update = True

    if keys[pygame.K_s]:
        camera.pos = camera.pos - camera.speed * camera.heading()
        update = True
    #moving forward and backward
    if keys[pygame.K_w]:
        camera.pos = camera.pos + camera.speed * camera.heading()
        update = True

    if keys[pygame.K_s]:
        camera.pos = camera.pos - camera.speed * camera.heading()
        update = True

    d = len(camera.pos)

    #holding alt translates rather than rotates the camera
    if keys[pygame.K_RALT] or keys[pygame.K_LALT]:
        for key_id in trans_keymapping[d]:
            if keys[key_id]:
                axis, trans_sign = trans_keymapping[d][key_id]
                camera.pos = camera.pos + trans_sign * camera.speed * camera.frame[
                    axis]
                update = True
    else:
        for key_id in rot_keymapping[d]:
            if keys[key_id]:
                axis1, axis2, angle_sign = rot_keymapping[d][key_id]
                camera.update_rot_matrix(axis1, axis2,
                                         angle_sign * camera.ang_speed)
                update = True

    #check for mouse motion events
    if enable_mouse:
        dmx, dmy = 0, 0
        #there can be many events here. which to choose?
        for event in events:
            if event.type == pygame.MOUSEMOTION:
                #print('mooovesd',event.pos)
                mx, my = event.pos
                dmx, dmy = mx - draw.center[0], my - draw.center[1]
                #break #choose first event
        if abs(dmx) > 2 or abs(dmy) > 2:
            #print(dmx,dmy)
            update = True
            pygame.mouse.set_pos(draw.center)
            if d == 3:
                camera.update_rot_matrix(
                    1, 2, -dmy / draw.height * 32 * camera.ang_speed)
                camera.update_rot_matrix(
                    0, 2, dmx / draw.height * 32 * camera.ang_speed)
            if d == 4:
                if keys[pygame.K_RSHIFT] or keys[pygame.K_LSHIFT]:
                    camera.update_rot_matrix(
                        2, 3, dmx / draw.height * 32 * camera.ang_speed)
                else:
                    camera.update_rot_matrix(
                        0, 3, dmx / draw.height * 32 * camera.ang_speed)
                camera.update_rot_matrix(
                    1, 3, -dmy / draw.height * 32 * camera.ang_speed)

    #any kind of whatever, update the plane
    if update:
        camera.update_plane()
    return update, quit