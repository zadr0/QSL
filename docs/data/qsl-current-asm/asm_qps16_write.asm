asm_qps16_write:
.Lfunc_begin6:
	.cv_func_id 44
	.cv_loc	44 5 29 0
.seh_proc asm_qps16_write
	pushq	%r15
	.seh_pushreg %r15
	pushq	%r14
	.seh_pushreg %r14
	pushq	%r13
	.seh_pushreg %r13
	pushq	%r12
	.seh_pushreg %r12
	pushq	%rsi
	.seh_pushreg %rsi
	pushq	%rdi
	.seh_pushreg %rdi
	pushq	%rbx
	.seh_pushreg %rbx
	pushq	%rax
	.seh_stackalloc 8
	.seh_endprologue
	movq	%rcx, %r8
.Ltmp71:
	.cv_file	12 "C:\\Users\\Guild\\.rustup\\toolchains\\stable-x86_64-pc-windows-msvc\\lib\\rustlib\\src\\rust\\library\\core\\src\\option.rs" "FC70106A756F98208E6A6A3E428310BDFA3531877CF6C6A99F39FFFA26AB2C40" 3
	.cv_inline_site_id 45 within 44 inlined_at 5 30 0
	.cv_inline_site_id 46 within 45 inlined_at 6 389 0
	.cv_inline_site_id 47 within 46 inlined_at 6 200 0
	.cv_loc	47 12 634 0
	movq	(%rcx), %rsi
.Ltmp72:
	.cv_loc	47 12 634 0
	movq	16(%rcx), %r10
.Ltmp73:
	.cv_loc	47 12 634 0
	movq	48(%rcx), %r11
.Ltmp74:
	.cv_loc	46 6 203 0
	leaq	(%rsi,%rsi), %rcx
.Ltmp75:
	leaq	(%rcx,%rcx,4), %rax
.Ltmp76:
	.cv_loc	46 6 206 0
	movl	%r11d, %r9d
	shll	$6, %r9d
	.cv_loc	46 6 203 0
	leal	(%rax,%r10,8), %eax
	.cv_loc	46 6 207 0
	addl	%r9d, %eax
	addl	$24, %eax
	shrl	$3, %eax
.Ltmp77:
	.cv_inline_site_id 48 within 45 inlined_at 6 389 0
	.cv_inline_site_id 49 within 48 inlined_at 6 307 0
	.cv_loc	49 12 634 0
	leaq	8(%r8), %rdi
.Ltmp78:
	.cv_loc	49 12 634 0
	leaq	24(%r8), %r9
.Ltmp79:
	.cv_loc	49 12 634 0
	movq	32(%r8), %rbx
.Ltmp80:
	.cv_loc	48 6 307 0
	leaq	(,%r11,8), %r14
	leaq	(%r14,%r10,2), %r14
	leaq	(%r14,%rbx,4), %r14
	orq	%rsi, %r14
.Ltmp81:
	.cv_loc	48 6 309 0
	movq	64(%r8), %rbx
.Ltmp82:
	.cv_loc	48 6 310 0
	addq	$10, %rbx
.Ltmp83:
	.cv_loc	48 6 310 0
	movq	%rbx, %r15
	shrq	$4, %r15
	leal	(%rbx,%rbx), %r12d
	andl	$30, %r12d
	orq	%r15, %r12
.Ltmp84:
	.cv_loc	48 6 311 0
	shlq	$8, %r12
.Ltmp85:
	orq	%r14, %r12
	shrq	$60, %rbx
.Ltmp86:
	.cv_loc	48 6 313 0
	movq	72(%r8), %r14
.Ltmp87:
	.cv_loc	48 6 314 0
	movq	%r14, %r13
	shrq	$4, %r13
	movl	%r14d, %r15d
	andl	$15, %r15d
	orq	%r13, %r15
.Ltmp88:
	.cv_loc	48 6 315 0
	shlq	$13, %r15
.Ltmp89:
	orq	%r12, %r15
	shrq	$55, %r14
.Ltmp90:
	orq	%rbx, %r14
.Ltmp91:
	.cv_loc	48 6 317 0
	movq	$0, (%rsp)
.Ltmp92:
	.cv_loc	48 6 317 0
	testq	%rsi, %rsi
	movq	%rsp, %rbx
.Ltmp93:
	.cv_inline_site_id 50 within 48 inlined_at 6 317 0
	.cv_loc	50 4 879 0
	cmoveq	%rbx, %rdi
.Ltmp94:
	.cv_inline_site_id 51 within 48 inlined_at 6 317 0
	.cv_loc	51 8 14 0
	#APP

	# rdi 

	#NO_APP
.Ltmp95:
	.cv_inline_site_id 52 within 48 inlined_at 6 317 0
	.cv_inline_site_id 53 within 52 inlined_at 11 1153 0
	.cv_loc	53 9 1731 0
	movq	(%rdi), %rdi
.Ltmp96:
	.cv_loc	48 6 318 0
	movq	%rdi, %r12
	shrq	$4, %r12
	movl	%edi, %esi
.Ltmp97:
	andl	$15, %esi
	shll	$6, %esi
	orq	%r12, %rsi
.Ltmp98:
	.cv_loc	48 6 319 0
	shlq	$17, %rsi
.Ltmp99:
	shrq	$51, %rdi
.Ltmp100:
	orq	%r15, %rsi
.Ltmp101:
	orq	%r14, %rdi
.Ltmp102:
	.cv_loc	48 6 321 0
	movq	$-100, (%rsp)
.Ltmp103:
	.cv_loc	48 6 321 0
	testq	%r10, %r10
.Ltmp104:
	.cv_inline_site_id 54 within 48 inlined_at 6 321 0
	.cv_loc	54 4 879 0
	cmoveq	%rbx, %r9
.Ltmp105:
	.cv_inline_site_id 55 within 48 inlined_at 6 321 0
	.cv_loc	55 8 14 0
	#APP

	# r9 

	#NO_APP
.Ltmp106:
	.cv_inline_site_id 56 within 48 inlined_at 6 321 0
	.cv_inline_site_id 57 within 56 inlined_at 11 1153 0
	.cv_loc	57 9 1731 0
	movq	(%r9), %r9
.Ltmp107:
	.cv_loc	48 6 322 0
	addq	$100, %r9
.Ltmp108:
	.cv_loc	48 6 322 0
	movq	%r9, %r15
	shrq	$4, %r15
	shll	$4, %r9d
.Ltmp109:
	movzbl	%r9b, %r14d
	orq	%r15, %r14
.Ltmp110:
	.cv_inline_site_id 58 within 48 inlined_at 6 323 0
	.cv_loc	58 6 196 0
	leaq	(%rcx,%rcx,4), %r9
	addq	$17, %r9
.Ltmp111:
	.cv_loc	48 6 323 0
	movq	%r14, %r15
	shrq	%r15
	movl	%r9d, %ecx
	notb	%cl
	shrq	%cl, %r15
	movl	%r9d, %ecx
	shlq	%cl, %r14
.Ltmp112:
	.cv_loc	49 12 634 0
	leaq	56(%r8), %rcx
.Ltmp113:
	.cv_loc	48 6 325 0
	movq	$0, (%rsp)
.Ltmp114:
	.cv_loc	48 6 325 0
	testq	%r11, %r11
.Ltmp115:
	.cv_inline_site_id 59 within 48 inlined_at 6 325 0
	.cv_loc	59 4 879 0
	cmoveq	%rbx, %rcx
.Ltmp116:
	.cv_inline_site_id 60 within 48 inlined_at 6 325 0
	.cv_loc	60 8 14 0
	#APP

	# rcx 

	#NO_APP
.Ltmp117:
	.cv_inline_site_id 61 within 48 inlined_at 6 325 0
	.cv_inline_site_id 62 within 61 inlined_at 11 1153 0
	.cv_loc	62 9 1731 0
	movq	(%rcx), %r8
.Ltmp118:
	.cv_loc	48 6 326 0
	rolq	$60, %r8
.Ltmp119:
	.cv_inline_site_id 63 within 48 inlined_at 6 327 0
	.cv_loc	63 6 198 0
	leal	(%r9,%r10,8), %ecx
.Ltmp120:
	.cv_loc	48 6 327 0
	movq	%r8, %r9
	shlq	%cl, %r9
	notb	%cl
	shrq	%r8
.Ltmp121:
	shrq	%cl, %r8
.Ltmp122:
	.cv_loc	48 6 323 0
	orq	%rdi, %r8
.Ltmp123:
	.cv_loc	48 6 327 0
	orq	%r15, %r8
.Ltmp124:
	.cv_loc	48 6 323 0
	orq	%rsi, %r9
.Ltmp125:
	.cv_loc	48 6 327 0
	orq	%r14, %r9
.Ltmp126:
	.cv_inline_site_id 64 within 48 inlined_at 6 328 0
	.cv_inline_site_id 65 within 64 inlined_at 10 1460 0
	.cv_inline_site_id 66 within 65 inlined_at 9 2032 0
	.cv_loc	66 9 1939 0
	movq	%r9, (%rdx)
	movq	%r8, 8(%rdx)
.Ltmp127:
	.cv_loc	44 5 31 0
	.seh_startepilogue
	addq	$8, %rsp
.Ltmp128:
	popq	%rbx
.Ltmp129:
	popq	%rdi
	popq	%rsi
	popq	%r12
	popq	%r13
	popq	%r14
	popq	%r15
	.seh_endepilogue
	retq
.Ltmp130:
.Lfunc_end6:
	.seh_endproc
