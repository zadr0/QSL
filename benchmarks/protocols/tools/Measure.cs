using System;
using System.Diagnostics;
using System.ComponentModel;
using System.Runtime.InteropServices;
public static class QslStrictMetrics {
 [StructLayout(LayoutKind.Sequential)] struct Basic {public long ProcessTime,JobTime; public uint Flags;public UIntPtr MinimumSet,MaximumSet;public uint Active;public UIntPtr Affinity;public uint Priority,Scheduling;}
 [StructLayout(LayoutKind.Sequential)] struct Io {public ulong A,B,C,D,E,F;}
 [StructLayout(LayoutKind.Sequential)] struct Extended {public Basic Basic;public Io Io;public UIntPtr ProcessLimit,JobLimit,PeakProcess,PeakJob;}
 [StructLayout(LayoutKind.Sequential)] struct Accounting {public long User,Kernel,PeriodUser,PeriodKernel;public uint PageFaults,Total,Active,Terminated;}
 [StructLayout(LayoutKind.Sequential)] struct Cpu {public long Idle,Kernel,User,Dpc,Interrupt;public uint InterruptCount;}
 [StructLayout(LayoutKind.Sequential)] struct Memory {public uint Size,PageFaults;public UIntPtr PeakWorkingSet,WorkingSet,PeakPaged,Paged,PeakNonpaged,Nonpaged,Pagefile,PeakPagefile,Private;}
 [DllImport("kernel32",CharSet=CharSet.Unicode,SetLastError=true)]static extern IntPtr CreateJobObject(IntPtr sec,string name);
 [DllImport("kernel32",SetLastError=true)]static extern bool SetInformationJobObject(IntPtr h,int type,ref Extended info,uint size);
 [DllImport("kernel32",SetLastError=true)]static extern bool QueryInformationJobObject(IntPtr h,int type,out Extended info,uint size,IntPtr returned);
 [DllImport("kernel32",EntryPoint="QueryInformationJobObject",SetLastError=true)]static extern bool QueryAccounting(IntPtr h,int type,out Accounting info,uint size,IntPtr returned);
 [DllImport("kernel32",SetLastError=true)]static extern bool AssignProcessToJobObject(IntPtr h,IntPtr p);
 [DllImport("kernel32")]public static extern bool CloseHandle(IntPtr h);
 [DllImport("kernel32")]static extern bool QueryProcessCycleTime(IntPtr h,out ulong cycles);
 [DllImport("psapi",SetLastError=true)]static extern bool GetProcessMemoryInfo(IntPtr h,out Memory info,uint size);
 [DllImport("ntdll")]static extern int NtQuerySystemInformation(int type,[Out]Cpu[] data,int length,out int returned);
 public static IntPtr CreateJob(){var h=CreateJobObject(IntPtr.Zero,null);if(h==IntPtr.Zero)throw new Win32Exception();var info=new Extended();info.Basic.Flags=0x200u|0x2000u;info.JobLimit=new UIntPtr(8UL*1024*1024*1024);if(!SetInformationJobObject(h,9,ref info,(uint)Marshal.SizeOf<Extended>()))throw new Win32Exception();return h;}
 public static void Attach(IntPtr h,IntPtr p){if(!AssignProcessToJobObject(h,p))throw new Win32Exception();}
 public static long[] CpuTimes(){var data=new Cpu[Environment.ProcessorCount];int returned;int code=NtQuerySystemInformation(8,data,data.Length*Marshal.SizeOf<Cpu>(),out returned);if(code!=0)throw new Exception("NtQuerySystemInformation: "+code);var result=new long[data.Length*3];for(int i=0;i<data.Length;i++){result[3*i]=data[i].Idle;result[3*i+1]=data[i].Kernel;result[3*i+2]=data[i].User;}return result;}
 public static double[] Busy(long[] a,long[] b){var result=new double[a.Length/3];for(int i=0;i<result.Length;i++){double total=(b[3*i+1]-a[3*i+1])+(b[3*i+2]-a[3*i+2]);result[i]=total<=0?0:100*(total-(b[3*i]-a[3*i]))/total;}return result;}
 public static double BusySeconds(long[] a,long[] b){double result=0;for(int i=0;i<a.Length;i+=3)result+=(b[i+1]-a[i+1])+(b[i+2]-a[i+2])-(b[i]-a[i]);return result/10000000.0;}
 public static double[] Finish(IntPtr job,IntPtr process){Extended e;Accounting a;ulong cycles;Memory memory;if(!QueryInformationJobObject(job,9,out e,(uint)Marshal.SizeOf<Extended>(),IntPtr.Zero)||!QueryAccounting(job,1,out a,(uint)Marshal.SizeOf<Accounting>(),IntPtr.Zero))throw new Win32Exception();QueryProcessCycleTime(process,out cycles);bool mem=GetProcessMemoryInfo(process,out memory,(uint)Marshal.SizeOf<Memory>());return new double[]{e.PeakJob.ToUInt64(),a.User/10000000.0,a.Kernel/10000000.0,cycles,mem?memory.PeakWorkingSet.ToUInt64():-1};}

 [StructLayout(LayoutKind.Sequential)] struct Time {public uint Low,High;public long Ticks{get{return ((long)High<<32)|Low;}}}
 [DllImport("kernel32",SetLastError=true)]static extern IntPtr OpenProcess(uint access,bool inherit,int pid);
 [DllImport("kernel32",SetLastError=true)]static extern bool GetProcessTimes(IntPtr process,out Time created,out Time exited,out Time kernel,out Time user);
 public sealed class ProcessWindow {
  internal System.Collections.Generic.Dictionary<int,IntPtr> Handles=new System.Collections.Generic.Dictionary<int,IntPtr>();
  internal System.Collections.Generic.Dictionary<int,long> Times=new System.Collections.Generic.Dictionary<int,long>();
  internal long CreatedAfter;public int Missing;
 }
 static bool ProcessTime(IntPtr h,out long time,out long created){Time c,e,k,u;bool ok=GetProcessTimes(h,out c,out e,out k,out u);time=k.Ticks+u.Ticks;created=c.Ticks;return ok;}
 public static ProcessWindow BeginProcesses(){var w=new ProcessWindow();w.CreatedAfter=DateTime.UtcNow.ToFileTimeUtc();foreach(var p in Process.GetProcesses()){try{int id=p.Id;if(id<=4||id==Environment.ProcessId)continue;var h=OpenProcess(0x1000,false,id);long t,c;if(h==IntPtr.Zero||!ProcessTime(h,out t,out c)){if(h!=IntPtr.Zero)CloseHandle(h);w.Missing++;continue;}w.Handles[id]=h;w.Times[id]=t;}catch{w.Missing++;}finally{p.Dispose();}}return w;}
 public static double[] EndProcesses(ProcessWindow w,int benchmarkPid){long ticks=0;int known=0,failed=w.Missing;try{foreach(var e in w.Handles){long t,c;if(ProcessTime(e.Value,out t,out c)){ticks+=Math.Max(0,t-w.Times[e.Key]);known++;}else failed++;}foreach(var p in Process.GetProcesses()){try{int id=p.Id;if(id<=4||id==Environment.ProcessId||id==benchmarkPid||w.Handles.ContainsKey(id))continue;var h=OpenProcess(0x1000,false,id);long t,c;if(h!=IntPtr.Zero){if(ProcessTime(h,out t,out c)&&c>=w.CreatedAfter){ticks+=t;known++;}CloseHandle(h);}}catch{failed++;}finally{p.Dispose();}}return new double[]{ticks/10000000.0,known,failed};}finally{foreach(var h in w.Handles.Values)CloseHandle(h);w.Handles.Clear();}}
 [StructLayout(LayoutKind.Sequential)]struct CounterValue{public uint Status;public double Value;}
 [DllImport("pdh",CharSet=CharSet.Unicode)]static extern uint PdhOpenQueryW(string source,UIntPtr user,out IntPtr query);
 [DllImport("pdh",CharSet=CharSet.Unicode)]static extern uint PdhAddEnglishCounterW(IntPtr query,string path,UIntPtr user,out IntPtr counter);
 [DllImport("pdh")]static extern uint PdhCollectQueryData(IntPtr query);
 [DllImport("pdh")]static extern uint PdhGetFormattedCounterValue(IntPtr counter,uint format,out uint type,out CounterValue value);
 [DllImport("pdh")]static extern uint PdhCloseQuery(IntPtr query);
 public sealed class KernelWindow{internal IntPtr Query;internal IntPtr Dpc,Interrupt,Switch;public bool Available;}
 public static KernelWindow BeginKernel(){var w=new KernelWindow();if(PdhOpenQueryW(null,UIntPtr.Zero,out w.Query)!=0)return w;w.Available=PdhAddEnglishCounterW(w.Query,@"\Processor(_Total)\% DPC Time",UIntPtr.Zero,out w.Dpc)==0&&PdhAddEnglishCounterW(w.Query,@"\Processor(_Total)\% Interrupt Time",UIntPtr.Zero,out w.Interrupt)==0&&PdhAddEnglishCounterW(w.Query,@"\System\Context Switches/sec",UIntPtr.Zero,out w.Switch)==0&&PdhCollectQueryData(w.Query)==0;return w;}
 static double Counter(IntPtr h){uint type;CounterValue v;return PdhGetFormattedCounterValue(h,0x200,out type,out v)==0&&v.Status<=1?v.Value:-1;}
 public static void MarkKernel(KernelWindow w){if(w.Available&&PdhCollectQueryData(w.Query)!=0)w.Available=false;}
 public static double[] EndKernel(KernelWindow w){try{if(!w.Available||PdhCollectQueryData(w.Query)!=0)return new double[]{-1,-1,-1};return new double[]{Counter(w.Dpc),Counter(w.Interrupt),Counter(w.Switch)};}finally{if(w.Query!=IntPtr.Zero)PdhCloseQuery(w.Query);}}
}
