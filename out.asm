add:
    addi sp, sp, -4
    sw   ra, 0(sp)
    addi sp, sp, -4
    sw   fp, 0(sp)
    addi fp, sp, 0
    lw   t0, 12(fp)
    addi sp, sp, -4
    sw   t0, 0(sp)
    lw   t0, 8(fp)
    addi sp, sp, -4
    sw   t0, 0(sp)
    lw   t1, 0(sp)
    addi sp, sp, 4
    lw   t0, 0(sp)
    addi sp, sp, 4
    add  t0, t0, t1
    addi sp, sp, -4
    sw   t0, 0(sp)
    lw   t0, 0(sp)
    addi sp, sp, 4
    lw   fp, 0(sp)
    addi sp, sp, 4
    lw   ra, 0(sp)
    addi sp, sp, 4
    jr   ra

main:
    addi sp, sp, -4
    sw   ra, 0(sp)
    addi sp, sp, -4
    sw   fp, 0(sp)
    addi fp, sp, 0
    addi t0, zero, 2
    addi sp, sp, -4
    sw   t0, 0(sp)
    addi t0, zero, 3
    addi sp, sp, -4
    sw   t0, 0(sp)
    bal add
    addi sp, sp, 8
    addi sp, sp, -4
    sw   t0, 0(sp)
    lw   t0, 0(sp)
    addi sp, sp, 4
    lw   fp, 0(sp)
    addi sp, sp, 4
    lw   ra, 0(sp)
    addi sp, sp, 4
    jr   ra
