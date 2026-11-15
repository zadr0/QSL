asm_raw_read:
.Lfunc_begin7:
	.cv_func_id 67
	.cv_loc	67 5 8 0
.seh_proc asm_raw_read
	.seh_handler __CxxFrameHandler3, @unwind, @except
	pushq	%rbp
	.seh_pushreg %rbp
	pushq	%rsi
	.seh_pushreg %rsi
	pushq	%rdi
	.seh_pushreg %rdi
	pushq	%rbx
	.seh_pushreg %rbx
	subq	$72, %rsp
	.seh_stackalloc 72
	leaq	64(%rsp), %rbp
	.seh_setframe %rbp, 64
	movaps	%xmm6, -16(%rbp)
	.seh_savexmm %xmm6, 48
	.seh_endprologue
	movq	$-2, -24(%rbp)
	movq	%rdx, %rax
.Ltmp133:
	.cv_inline_site_id 68 within 67 inlined_at 5 9 0
	.cv_inline_site_id 69 within 68 inlined_at 6 22 0
	.cv_file	13 "C:\\Users\\Guild\\Desktop\\dev-qsl\\qsl\\src\\serialize.rs" "AE717E8CFF45FE5AF045030A8C8DF6F7B6AB1B93BEDB534E9F73A9B8D2FF8FBB" 3
	.cv_inline_site_id 70 within 69 inlined_at 13 403 0
	.cv_inline_site_id 71 within 70 inlined_at 13 90 0
	.cv_inline_site_id 72 within 71 inlined_at 6 176 0
	.cv_inline_site_id 73 within 72 inlined_at 6 150 0
	.cv_loc	73 6 61 0
	xorl	%edx, %edx
.Ltmp134:
	addq	$-17, %rax
.Ltmp135:
	setne	%dl
	cmpq	$8, %rax
	sbbq	$-1, %rdx
	cmpq	$9, %rax
	sbbq	$-1, %rdx
	cmpq	$16, %rax
	sbbq	$-1, %rdx
	movq	%r8, %rsi
	cmpq	$17, %rax
	sbbq	$-1, %rdx
	movq	%rcx, %rdi
	cmpq	$24, %rax
	sbbq	$-1, %rdx
.Ltmp136:
	.cv_inline_site_id 74 within 72 inlined_at 6 148 0
	.cv_file	14 "C:\\Users\\Guild\\Desktop\\dev-qsl\\qsl\\src\\bit_io.rs" "E98566E53BE655B912CAC0DEFC65CF4F30CB351A07E0FA1FAD0303013B24CBCB" 3
	.cv_inline_site_id 75 within 74 inlined_at 14 170 0
	.cv_inline_site_id 76 within 75 inlined_at 14 146 0
	.cv_inline_site_id 77 within 76 inlined_at 9 1826 0
	.cv_loc	77 9 1731 0
	movups	1(%rcx), %xmm6
.Ltmp137:
	.cv_loc	73 6 61 0
	cmpq	$25, %rax
	sbbq	$-1, %rdx
	cmpq	$32, %rax
	sbbq	$-1, %rdx
.Ltmp138:
	.cv_file	15 "C:\\Users\\Guild\\.rustup\\toolchains\\stable-x86_64-pc-windows-msvc\\lib\\rustlib\\src\\rust\\library\\core\\src\\slice\\index.rs" "8C2D4F2DD29808136D4A45878010F2EDB7C64C3C197D564F86D034B33DF4E8EC" 3
	.cv_inline_site_id 78 within 72 inlined_at 6 151 0
	.cv_file	16 "C:\\Users\\Guild\\.rustup\\toolchains\\stable-x86_64-pc-windows-msvc\\lib\\rustlib\\src\\rust\\library\\core\\src\\slice\\mod.rs" "453BFD6E0FC09E06AC1793CCB03E2E2F04FAE149DB6A079256097C8FAD510669" 3
	.cv_inline_site_id 79 within 78 inlined_at 16 647 0
	.cv_loc	79 15 219 0
	movq	__imp__RNvCs4RZz9i8IIt6_17qsl_current_bench15Raw_EVENT_TABLE(%rip), %rax
.Ltmp139:
	.cv_loc	72 6 151 0
.Ltmp131:
	callq	*(%rax,%rdx,8)
	nop
.Ltmp140:
.Ltmp132:
	.cv_inline_site_id 80 within 72 inlined_at 6 149 0
	.cv_inline_site_id 81 within 80 inlined_at 14 146 0
	.cv_inline_site_id 82 within 81 inlined_at 14 121 0
	.cv_loc	82 11 872 0
	leaq	17(%rdi), %rdx
.Ltmp141:
	.cv_loc	72 6 152 0
	movl	%eax, %ecx
	andl	$1, %ecx
.Ltmp142:
	.cv_loc	72 6 152 0
	movq	$0, -32(%rbp)
	leaq	-32(%rbp), %r8
.Ltmp143:
	.cv_inline_site_id 83 within 72 inlined_at 6 152 0
	.cv_loc	83 4 879 0
	cmoveq	%r8, %rdx
.Ltmp144:
	.cv_inline_site_id 84 within 72 inlined_at 6 152 0
	.cv_loc	84 8 14 0
	#APP

	# rdx 

	#NO_APP
.Ltmp145:
	.cv_inline_site_id 85 within 72 inlined_at 6 152 0
	.cv_inline_site_id 86 within 85 inlined_at 9 1826 0
	.cv_loc	86 9 1731 0
	movq	(%rdx), %rdx
.Ltmp146:
	.cv_loc	85 8 29 0
	#APP

	# rdx 

	#NO_APP
.Ltmp147:
	.cv_loc	72 6 152 0
	movl	%ecx, %r9d
.Ltmp148:
	.cv_inline_site_id 87 within 72 inlined_at 6 152 0
	.cv_inline_site_id 88 within 87 inlined_at 14 121 0
	.cv_loc	88 11 872 0
	leaq	(%rdi,%r9,8), %r10
	addq	$17, %r10
.Ltmp149:
	.cv_loc	72 6 153 0
	movl	%eax, %r9d
	andl	$2, %r9d
.Ltmp150:
	.cv_loc	72 6 153 0
	movq	$0, -32(%rbp)
.Ltmp151:
	.cv_loc	72 6 153 0
	movl	%r9d, %r11d
.Ltmp152:
	.cv_inline_site_id 89 within 72 inlined_at 6 153 0
	.cv_inline_site_id 90 within 89 inlined_at 14 121 0
	.cv_loc	90 11 872 0
	leaq	(%r10,%r11,4), %rdi
.Ltmp153:
	.cv_inline_site_id 91 within 72 inlined_at 6 153 0
	.cv_loc	91 4 879 0
	cmoveq	%r8, %r10
.Ltmp154:
	.cv_inline_site_id 92 within 72 inlined_at 6 153 0
	.cv_loc	92 8 14 0
	#APP

	# r10 

	#NO_APP
.Ltmp155:
	.cv_inline_site_id 93 within 72 inlined_at 6 153 0
	.cv_inline_site_id 94 within 93 inlined_at 9 1826 0
	.cv_loc	94 9 1731 0
	movq	(%r10), %r10
.Ltmp156:
	.cv_loc	93 8 29 0
	#APP

	# r10 

	#NO_APP
.Ltmp157:
	.cv_loc	89 7 37 0
	shrl	%r9d
.Ltmp158:
	.cv_loc	72 6 154 0
	movl	%eax, %r11d
	andl	$4, %r11d
.Ltmp159:
	.cv_loc	72 6 154 0
	movq	$0, -32(%rbp)
.Ltmp160:
	.cv_loc	72 6 154 0
	movl	%r11d, %ebx
.Ltmp161:
	.cv_inline_site_id 95 within 72 inlined_at 6 154 0
	.cv_inline_site_id 96 within 95 inlined_at 14 121 0
	.cv_loc	96 11 872 0
	leaq	(%rdi,%rbx,2), %rbx
.Ltmp162:
	.cv_inline_site_id 97 within 72 inlined_at 6 154 0
	.cv_loc	97 4 879 0
	cmoveq	%r8, %rdi
.Ltmp163:
	.cv_inline_site_id 98 within 72 inlined_at 6 154 0
	.cv_loc	98 8 14 0
	#APP

	# rdi 

	#NO_APP
.Ltmp164:
	.cv_inline_site_id 99 within 72 inlined_at 6 154 0
	.cv_inline_site_id 100 within 99 inlined_at 9 1826 0
	.cv_loc	100 9 1731 0
	movq	(%rdi), %rdi
.Ltmp165:
	.cv_loc	99 8 29 0
	#APP

	# rdi 

	#NO_APP
.Ltmp166:
	.cv_loc	95 7 37 0
	shrl	$2, %r11d
.Ltmp167:
	.cv_loc	72 6 155 0
	andl	$8, %eax
.Ltmp168:
	.cv_loc	72 6 155 0
	movq	$0, -32(%rbp)
.Ltmp169:
	.cv_inline_site_id 101 within 72 inlined_at 6 155 0
	.cv_loc	101 4 879 0
	cmoveq	%r8, %rbx
.Ltmp170:
	.cv_inline_site_id 102 within 72 inlined_at 6 155 0
	.cv_loc	102 8 14 0
	#APP

	# rbx 

	#NO_APP
.Ltmp171:
	.cv_inline_site_id 103 within 72 inlined_at 6 155 0
	.cv_inline_site_id 104 within 103 inlined_at 9 1826 0
	.cv_loc	104 9 1731 0
	movq	(%rbx), %r8
.Ltmp172:
	.cv_loc	103 8 29 0
	#APP

	# r8 

	#NO_APP
.Ltmp173:
	.cv_inline_site_id 105 within 72 inlined_at 6 155 0
	.cv_loc	105 7 37 0
	shrl	$3, %eax
.Ltmp174:
	.cv_inline_site_id 106 within 71 inlined_at 10 1400 0
	.cv_loc	106 9 1939 0
	movq	%rcx, (%rsi)
	movq	%rdx, 8(%rsi)
	movq	%r9, 16(%rsi)
	movq	%r10, 24(%rsi)
	movq	%r11, 32(%rsi)
	movq	%rdi, 40(%rsi)
	movq	%rax, 48(%rsi)
	movq	%r8, 56(%rsi)
	movups	%xmm6, 64(%rsi)
	movaps	-16(%rbp), %xmm6
.Ltmp175:
	.cv_loc	67 5 10 0
	.seh_startepilogue
	addq	$72, %rsp
	popq	%rbx
.Ltmp176:
	popq	%rdi
.Ltmp177:
	popq	%rsi
.Ltmp178:
	popq	%rbp
	.seh_endepilogue
	retq
.Ltmp179:
	.seh_handlerdata
	.long	$cppxdata$asm_raw_read@IMGREL
	.section	.text,"xr",one_only,asm_raw_read,unique,7
	.seh_endproc