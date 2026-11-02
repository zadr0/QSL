asm_raw_write:
.Lfunc_begin8:
	.cv_func_id 107
	.cv_loc	107 5 15 0
.seh_proc asm_raw_write
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
	subq	$16, %rsp
	.seh_stackalloc 16
	.seh_endprologue
.Ltmp181:
	.cv_inline_site_id 108 within 107 inlined_at 5 16 0
	.cv_inline_site_id 109 within 108 inlined_at 6 172 0
	.cv_inline_site_id 110 within 109 inlined_at 6 138 0
	.cv_loc	110 12 634 0
	leaq	8(%rcx), %r8
	movq	(%rcx), %r14
.Ltmp182:
	.cv_loc	110 12 634 0
	movq	16(%rcx), %r15
.Ltmp183:
	.cv_loc	110 12 634 0
	movq	32(%rcx), %r11
.Ltmp184:
	.cv_loc	109 6 138 0
	leal	(,%r11,4), %r9d
.Ltmp185:
	.cv_loc	110 12 634 0
	movq	48(%rcx), %rax
.Ltmp186:
	.cv_loc	109 6 138 0
	leaq	(,%rax,8), %rsi
	leal	(%r9,%r15,2), %r9d
	addl	%r14d, %r9d
	orl	%r9d, %esi
.Ltmp187:
	.cv_file	17 "C:\\Users\\Guild\\.rustup\\toolchains\\stable-x86_64-pc-windows-msvc\\lib\\rustlib\\src\\rust\\library\\core\\src\\macros\\mod.rs" "9784131E43D255FED7A07AE18CBC72748EEDBEBE65DFCEFEEE999989289DF30C" 3
	.cv_loc	110 17 430 0
	testq	%r14, %r14
	movq	%rsp, %r10
.Ltmp188:
	.cv_inline_site_id 111 within 109 inlined_at 6 141 0
	.cv_loc	111 4 879 0
	cmoveq	%r10, %r8
.Ltmp189:
	.cv_inline_site_id 112 within 109 inlined_at 6 140 0
	.cv_inline_site_id 113 within 112 inlined_at 14 235 0
	.cv_inline_site_id 114 within 113 inlined_at 14 241 0
	.cv_inline_site_id 115 within 114 inlined_at 14 228 0
	.cv_loc	115 10 970 0
	leaq	17(%rdx), %rdi
	leaq	8(%rsp), %r9
.Ltmp190:
	.cv_inline_site_id 116 within 109 inlined_at 6 141 0
	.cv_loc	116 4 879 0
	cmoveq	%r9, %rdi
.Ltmp191:
	.cv_loc	110 12 634 0
	leaq	24(%rcx), %rbx
.Ltmp192:
	.cv_loc	110 17 430 0
	testq	%r15, %r15
.Ltmp193:
	.cv_inline_site_id 117 within 109 inlined_at 6 142 0
	.cv_loc	117 4 879 0
	cmoveq	%r10, %rbx
.Ltmp194:
	.cv_inline_site_id 118 within 109 inlined_at 6 141 0
	.cv_inline_site_id 119 within 118 inlined_at 14 228 0
	.cv_loc	119 10 970 0
	leaq	17(%rdx,%r14,8), %r14
.Ltmp195:
	.cv_inline_site_id 120 within 109 inlined_at 6 142 0
	.cv_inline_site_id 121 within 120 inlined_at 14 228 0
	.cv_loc	121 10 970 0
	leaq	(%r14,%r15,8), %r12
.Ltmp196:
	.cv_inline_site_id 122 within 109 inlined_at 6 142 0
	.cv_loc	122 4 879 0
	cmoveq	%r9, %r14
.Ltmp197:
	.cv_loc	110 12 634 0
	leaq	40(%rcx), %r15
.Ltmp198:
	.cv_loc	110 17 430 0
	testq	%r11, %r11
.Ltmp199:
	.cv_inline_site_id 123 within 109 inlined_at 6 143 0
	.cv_loc	123 4 879 0
	cmoveq	%r10, %r15
.Ltmp200:
	.cv_inline_site_id 124 within 109 inlined_at 6 143 0
	.cv_inline_site_id 125 within 124 inlined_at 14 228 0
	.cv_loc	125 10 970 0
	leaq	(%r12,%r11,8), %r11
.Ltmp201:
	.cv_inline_site_id 126 within 109 inlined_at 6 143 0
	.cv_loc	126 4 879 0
	cmoveq	%r9, %r12
.Ltmp202:
	.cv_loc	110 12 634 0
	leaq	56(%rcx), %r13
.Ltmp203:
	.cv_loc	110 17 430 0
	testq	%rax, %rax
.Ltmp204:
	.cv_inline_site_id 127 within 109 inlined_at 6 138 0
	.cv_inline_site_id 128 within 127 inlined_at 14 235 0
	.cv_inline_site_id 129 within 128 inlined_at 14 242 0
	.cv_loc	129 9 549 0
	movb	%sil, (%rdx)
.Ltmp205:
	.cv_loc	109 6 139 0
	movups	64(%rcx), %xmm0
.Ltmp206:
	.cv_inline_site_id 130 within 109 inlined_at 6 139 0
	.cv_inline_site_id 131 within 130 inlined_at 14 235 0
	.cv_inline_site_id 132 within 131 inlined_at 14 242 0
	.cv_loc	132 9 549 0
	movups	%xmm0, 1(%rdx)
.Ltmp207:
	.cv_loc	109 6 141 0
	movq	$0, (%rsp)
.Ltmp208:
	.cv_inline_site_id 133 within 109 inlined_at 6 141 0
	.cv_loc	133 8 14 0
	#APP

	# r8 

	#NO_APP
.Ltmp209:
	.cv_inline_site_id 134 within 109 inlined_at 6 141 0
	.cv_loc	134 9 1731 0
	movq	(%r8), %rcx
.Ltmp210:
	.cv_inline_site_id 135 within 109 inlined_at 6 141 0
	.cv_loc	135 8 14 0
	#APP

	# rdi 

	#NO_APP
.Ltmp211:
	.cv_inline_site_id 136 within 118 inlined_at 9 2032 0
	.cv_loc	136 9 1939 0
	movq	%rcx, (%rdi)
.Ltmp212:
	.cv_loc	109 6 142 0
	movq	$0, (%rsp)
.Ltmp213:
	.cv_inline_site_id 137 within 109 inlined_at 6 142 0
	.cv_loc	137 8 14 0
	#APP

	# rbx 

	#NO_APP
.Ltmp214:
	.cv_inline_site_id 138 within 109 inlined_at 6 142 0
	.cv_loc	138 9 1731 0
	movq	(%rbx), %rcx
.Ltmp215:
	.cv_inline_site_id 139 within 109 inlined_at 6 142 0
	.cv_loc	139 8 14 0
	#APP

	# r14 

	#NO_APP
.Ltmp216:
	.cv_inline_site_id 140 within 120 inlined_at 9 2032 0
	.cv_loc	140 9 1939 0
	movq	%rcx, (%r14)
.Ltmp217:
	.cv_loc	109 6 143 0
	movq	$0, (%rsp)
.Ltmp218:
	.cv_inline_site_id 141 within 109 inlined_at 6 143 0
	.cv_loc	141 8 14 0
	#APP

	# r15 

	#NO_APP
.Ltmp219:
	.cv_inline_site_id 142 within 109 inlined_at 6 143 0
	.cv_loc	142 9 1731 0
	movq	(%r15), %rcx
.Ltmp220:
	.cv_inline_site_id 143 within 109 inlined_at 6 143 0
	.cv_loc	143 8 14 0
	#APP

	# r12 

	#NO_APP
.Ltmp221:
	.cv_inline_site_id 144 within 124 inlined_at 9 2032 0
	.cv_loc	144 9 1939 0
	movq	%rcx, (%r12)
.Ltmp222:
	.cv_loc	109 6 144 0
	movq	$0, (%rsp)
.Ltmp223:
	.cv_inline_site_id 145 within 109 inlined_at 6 144 0
	.cv_loc	145 4 879 0
	cmoveq	%r10, %r13
.Ltmp224:
	.cv_inline_site_id 146 within 109 inlined_at 6 144 0
	.cv_loc	146 8 14 0
	#APP

	# r13 

	#NO_APP
.Ltmp225:
	.cv_inline_site_id 147 within 109 inlined_at 6 144 0
	.cv_loc	147 9 1731 0
	movq	(%r13), %rcx
.Ltmp226:
	.cv_inline_site_id 148 within 109 inlined_at 6 144 0
	.cv_loc	148 4 879 0
	cmovneq	%r11, %r9
.Ltmp227:
	.cv_inline_site_id 149 within 109 inlined_at 6 144 0
	.cv_loc	149 8 14 0
	#APP

	# r9 

	#NO_APP
.Ltmp228:
	.cv_inline_site_id 150 within 109 inlined_at 6 144 0
	.cv_inline_site_id 151 within 150 inlined_at 9 2032 0
	.cv_loc	151 9 1939 0
	movq	%rcx, (%r9)
.Ltmp229:
	.cv_inline_site_id 152 within 150 inlined_at 14 228 0
	.cv_loc	152 10 970 0
	leaq	(%r11,%rax,8), %rax
.Ltmp230:
	.cv_inline_site_id 153 within 109 inlined_at 6 145 0
	.cv_inline_site_id 154 within 153 inlined_at 10 803 0
	.cv_loc	154 11 624 0
	subq	%rdx, %rax
.Ltmp231:
	.cv_loc	107 5 17 0
	.seh_startepilogue
	addq	$16, %rsp
.Ltmp232:
	popq	%rbx
.Ltmp233:
	popq	%rdi
.Ltmp234:
	popq	%rsi
	popq	%r12
.Ltmp235:
	popq	%r13
.Ltmp236:
	popq	%r14
.Ltmp237:
	popq	%r15
.Ltmp238:
	.seh_endepilogue
	retq
.Ltmp239:
.Lfunc_end8:
	.seh_endproc
