.PHONY: clean

CC ?= gcc
OBJCOPY ?= objcopy

CARGO := cargo
GUEST_ASM := guest.S
GUEST_OBJ := $(patsubst %.S,%.o,$(GUEST_ASM))
GUEST_TARGET := $(patsubst %.S,%.bin,$(GUEST_ASM))

run: $(GUEST_TARGET)
	$(CARGO) run --release

$(GUEST_TARGET): $(GUEST_OBJ)
	$(OBJCOPY) --only-section=.text -O binary $< $@

$(GUEST_OBJ): $(GUEST_ASM)
	$(CC) -c $< -o $@

clean:
	rm -f $(GUEST_OBJ) $(GUEST_TARGET)

