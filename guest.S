# Assembly for the guest to run

.code16 # 16-bit code

.text
.globl _start
_start:
	xor %ax, %ax      # Clear ax to 0
	mov $10, %cx      # Set loop counter to 10
loop_start:
	inc %ax           # Increment ax
	dec %cx           # Decrement cx
	jnz loop_start    # Jump back if cx != 0
	nop
	hlt               # Halt the VM

