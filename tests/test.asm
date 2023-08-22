bits 64

%macro SYMBOL 1
    global __symbol%1
    __symbol%1:
	nop
%endmacro

%assign i 0
%rep 100000
   SYMBOL i
%assign i i+1
%endrep

