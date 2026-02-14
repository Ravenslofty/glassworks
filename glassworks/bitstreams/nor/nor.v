(* blackbox *)
module NR2(input A, B, output QN);
endmodule

(* blackbox *)
module IPBUF (input EXTIN, output I);
endmodule

(* blackbox *)
module OPBUF (input O, output EXTOUT);
endmodule

module top(input A, B, output Q);
wire a_, b_, q_;
(* DPLD_PAD_PLACE="N3" *)
IPBUF ai (.EXTIN(A), .I(a_));
(* DPLD_PAD_PLACE="R3" *)
IPBUF bi (.EXTIN(B), .I(b_));
(* DPLD_PAD_PLACE="N4" *)
OPBUF qo (.O(q_), .EXTOUT(Q));
NR2 an2 (.A(a_), .B(b_), .QN(q_));
endmodule
