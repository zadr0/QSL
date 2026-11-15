asm_qps16_read:
.Lfunc_begin5:
	.cv_func_id 10
	.cv_loc	10 5 22 0
	.cv_file	6 "C:\\Users\\Guild\\Desktop\\dev-qsl\\qsl-bench\\current\\target\\release\\build\\qsl-current-bench-958091c5fc9af164\\out\\qsl_gen.rs" "DF67F4B7697811DCC976C39C1135DBC4E6034A473903A197CCF2A73F01851A62" 3
	.cv_inline_site_id 11 within 10 inlined_at 5 23 0
	.cv_inline_site_id 12 within 11 inlined_at 6 389 0
	.cv_loc	12 6 335 0
	movzbl	1(%rcx), %eax
.Ltmp25:
	movl	%eax, %r9d
	andl	$30, %r9d
.Ltmp26:
	.cv_loc	12 6 336 0
	shll	$4, %eax
	andl	$16, %eax
	shrl	%r9d
.Ltmp27:
	orl	%eax, %r9d
.Ltmp28:
	.cv_loc	12 6 336 0
	addq	$-10, %r9
.Ltmp29:
	.cv_file	7 "C:\\Users\\Guild\\.rustup\\toolchains\\stable-x86_64-pc-windows-msvc\\lib\\rustlib\\src\\rust\\library\\core\\src\\bool.rs" "1696AE209C49FFB85636219ECBBA1D12320960F8550543818CA39C7B26792711" 3
	.cv_inline_site_id 13 within 12 inlined_at 6 361 0
	.cv_loc	13 7 37 0
	movzbl	(%rcx), %edx
.Ltmp30:
	.cv_file	8 "C:\\Users\\Guild\\Desktop\\dev-qsl\\qsl\\src\\payload_quant.rs" "06D21EE181B85282DD3734253B35A77B61F2DB55A6F59FE02E79A3FBC8D75947" 3
	.cv_inline_site_id 14 within 12 inlined_at 6 337 0
	.cv_loc	14 8 29 0
	#APP

	# r9 

	#NO_APP
.Ltmp31:
	.cv_file	9 "C:\\Users\\Guild\\.rustup\\toolchains\\stable-x86_64-pc-windows-msvc\\lib\\rustlib\\src\\rust\\library\\core\\src\\ptr\\mod.rs" "F7B3E71D9E9246762C38641C6C1A49079265782AAB6DEAA537B43665B418F029" 3
	.cv_inline_site_id 15 within 12 inlined_at 6 337 0
	.cv_file	10 "C:\\Users\\Guild\\.rustup\\toolchains\\stable-x86_64-pc-windows-msvc\\lib\\rustlib\\src\\rust\\library\\core\\src\\ptr\\mut_ptr.rs" "54527A10B3C1B7BD589C13443EB2482BA8B8D9004B8184D58BB04A553345C1A9" 3
	.cv_inline_site_id 16 within 15 inlined_at 10 1400 0
	.cv_loc	16 9 1939 0
	movq	%r9, 64(%r8)
.Ltmp32:
	.cv_inline_site_id 17 within 12 inlined_at 6 340 0
	.cv_file	11 "C:\\Users\\Guild\\.rustup\\toolchains\\stable-x86_64-pc-windows-msvc\\lib\\rustlib\\src\\rust\\library\\core\\src\\ptr\\const_ptr.rs" "5CA8F4E779447933814DD78CF23E858064A395C058F034422C30AFC18E68C16C" 3
	.cv_inline_site_id 18 within 17 inlined_at 11 1194 0
	.cv_inline_site_id 19 within 18 inlined_at 9 1826 0
	.cv_loc	19 9 1731 0
	movzwl	1(%rcx), %eax
.Ltmp33:
	.cv_loc	12 6 340 0
	shrl	$5, %eax
	andl	$15, %eax
.Ltmp34:
	.cv_inline_site_id 20 within 12 inlined_at 6 342 0
	.cv_loc	20 8 29 0
	#APP

	# rax 

	#NO_APP
.Ltmp35:
	.cv_inline_site_id 21 within 12 inlined_at 6 342 0
	.cv_inline_site_id 22 within 21 inlined_at 10 1400 0
	.cv_loc	22 9 1939 0
	movq	%rax, 72(%r8)
.Ltmp36:
	.cv_loc	12 6 344 0
	movl	%edx, %eax
.Ltmp37:
	andl	$1, %eax
.Ltmp38:
	.cv_inline_site_id 23 within 12 inlined_at 6 345 0
	.cv_inline_site_id 24 within 23 inlined_at 11 1194 0
	.cv_inline_site_id 25 within 24 inlined_at 9 1826 0
	.cv_loc	25 9 1731 0
	movzwl	2(%rcx), %r9d
.Ltmp39:
	.cv_loc	12 6 346 0
	movl	%r9d, %r10d
	andl	$126, %r10d
	shrl	$7, %r9d
	andl	$15, %r9d
	leal	(%r9,%r10,8), %r9d
.Ltmp40:
	.cv_inline_site_id 26 within 12 inlined_at 6 347 0
	.cv_loc	26 8 29 0
	#APP

	# r9 

	#NO_APP
.Ltmp41:
	.cv_inline_site_id 27 within 12 inlined_at 6 347 0
	.cv_inline_site_id 28 within 27 inlined_at 10 1400 0
	.cv_loc	28 9 1939 0
	movq	%rax, (%r8)
	movq	%r9, 8(%r8)
.Ltmp42:
	.cv_inline_site_id 29 within 12 inlined_at 6 348 0
	.cv_loc	29 6 196 0
	addl	%eax, %eax
.Ltmp43:
	leal	(%rax,%rax,4), %eax
	addl	$17, %eax
.Ltmp44:
	.cv_loc	12 6 349 0
	movl	%edx, %r9d
.Ltmp45:
	shrl	%r9d
	andl	$1, %r9d
.Ltmp46:
	.cv_loc	12 6 350 0
	movl	%eax, %r11d
	shrl	$3, %r11d
.Ltmp47:
	.cv_inline_site_id 30 within 12 inlined_at 6 350 0
	.cv_loc	30 11 872 0
	leaq	(%rcx,%r11), %r10
.Ltmp48:
	.cv_inline_site_id 31 within 30 inlined_at 11 1194 0
	.cv_inline_site_id 32 within 31 inlined_at 9 1826 0
	.cv_loc	32 9 1731 0
	movzwl	(%rcx,%r11), %r11d
.Ltmp49:
	.cv_loc	12 6 350 0
	andb	$7, %al
	movl	%eax, %ecx
.Ltmp50:
	shrq	%cl, %r11
.Ltmp51:
	.cv_loc	12 6 351 0
	movl	%r11d, %ecx
	shll	$4, %ecx
	movzbl	%cl, %ecx
	shrl	$4, %r11d
.Ltmp52:
	andl	$15, %r11d
.Ltmp53:
	orl	%ecx, %r11d
.Ltmp54:
	.cv_loc	12 6 351 0
	addq	$-100, %r11
.Ltmp55:
	.cv_inline_site_id 33 within 12 inlined_at 6 352 0
	.cv_loc	33 8 29 0
	#APP

	# r11 

	#NO_APP
.Ltmp56:
	.cv_inline_site_id 34 within 12 inlined_at 6 352 0
	.cv_inline_site_id 35 within 34 inlined_at 10 1400 0
	.cv_loc	35 9 1939 0
	movq	%r9, 16(%r8)
	movq	%r11, 24(%r8)
.Ltmp57:
	.cv_inline_site_id 36 within 12 inlined_at 6 356 0
	.cv_loc	36 8 29 0
	movl	$7, %ecx
	#APP

	# rcx 

	#NO_APP
.Ltmp58:
	.cv_inline_site_id 37 within 12 inlined_at 6 356 0
	.cv_loc	37 7 37 0
	movl	%edx, %r11d
.Ltmp59:
	andl	$4, %r11d
	shrl	$2, %r11d
.Ltmp60:
	.cv_inline_site_id 38 within 37 inlined_at 10 1400 0
	.cv_loc	38 9 1939 0
	movq	%r11, 32(%r8)
	movq	%rcx, 40(%r8)
.Ltmp61:
	.cv_inline_site_id 39 within 12 inlined_at 6 359 0
	.cv_inline_site_id 40 within 39 inlined_at 11 1194 0
	.cv_inline_site_id 41 within 40 inlined_at 9 1826 0
	.cv_loc	41 9 1731 0
	movq	(%r9,%r10), %r11
.Ltmp62:
	movq	8(%r9,%r10), %r9
.Ltmp63:
	.cv_loc	12 6 359 0
	movl	%eax, %ecx
.Ltmp64:
	shrdq	%cl, %r9, %r11
.Ltmp65:
	.cv_loc	12 6 360 0
	rolq	$4, %r11
.Ltmp66:
	.cv_inline_site_id 42 within 12 inlined_at 6 361 0
	.cv_loc	42 8 29 0
	#APP

	# r11 

	#NO_APP
.Ltmp67:
	.cv_loc	13 7 37 0
	andl	$8, %edx
	shrl	$3, %edx
.Ltmp68:
	.cv_inline_site_id 43 within 13 inlined_at 10 1400 0
	.cv_loc	43 9 1939 0
	movq	%rdx, 48(%r8)
	movq	%r11, 56(%r8)
.Ltmp69:
	.cv_loc	10 5 24 0
	retq
.Ltmp70:
.Lfunc_end5:
