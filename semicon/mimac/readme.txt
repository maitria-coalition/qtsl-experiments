
SUBJECT: Semiconductor wafer fab datasets available.

SUMMARY:  
A set of factory-level datasets describing several semiconductor wafer
fabrication facilities is available from this site and via anonymous ftp. 
The datasets have been made available in order to facilitate the
evaluation and comparison of simulation tools, analytic tools, and fab
control strategies.

OVERVIEW:  
There currently exist a wide variety of commercial, corporate, and
university developed simulation and analytical tools for the analysis of
wafer fabs. There have also been a number of strategies and rules
developed to control the flow of product in a wafer fab.  However, it has
been difficult for semiconductor manufacturers, as well as for suppliers
and researchers, to evaluate and compare these tools and strategies. One
of the major problems in such comparisons has been the lack of
representative data. 
The purpose of this effort was to provide planners, researchers and
software suppliers with actual data and models that can be used to
benchmark their control strategies and software. The data set format was
developed by Gerry Feigin (IBM), John Fowler (SEMATECH-now ASU), and
Robert Leachman (University of California- Berkeley). The datasets are
available to anyone.


DETAILS -- FACTORY LEVEL DATASETS
The factory level datasets are available on this site and via anonymous
ftp from 'ftp.sematech.org' in directory '/pub/datasets'. The testbed
includes actual manufacturing data from several wafer fabrication
facilities, organized into a standard format. The data sets do not include
real product names, company names or other nomenclature that could serve
to identify the source of the data.  The datasets are as nonsensitive as
possible, so that they may be widely circulated.

The goal in development of the fab level data sets was to include the
minimum information necessary to model a factory. In support of this goal,
the data sets were kept as simple as possible, specifically by including
data but not modeling assumptions. For example, mean times between
failures are provided, but not their distributions. Process holds,
engineering holds, and send aheads are also not included. The resulting
data is given in six ASCII flat files, containing: product routings and
processing times; rework routings; equipment availability; operator
availability; product starts; and a comments file.

The testbed currently contains 6 factory-level datasets. Some
general information about the datasets follows:

Dataset       Type of Factory      # of Products         Volume
                                                    (wafer starts/mo)
Set1          Non-volatile memory       2                16,000
Set2          ASIC & Memory             7                10,000
Set3          Memory, various types    11                21,400
Set4          Microprocessors           7                 3,400
Set5          ASIC                    177                10,000  
Set6          ASIC                      9                 5,500

More information is included in the comments file provided with each
dataset. The comments files all have the ending .cf (e.g.  set1.cf). The
comments files each include a revision number for the dataset, and a list
of changes made since the last revision was published. 

For information on the testbed dataset format, download testbed.ps
(postscript version) or testbed.tex (TEX version) from the main directory. 
Each dataset is in its own directory. One may either download the .zip
file(which contains a compressed version of all the other files) or each
of the individual files. There is also a copy of an Excel spreadsheet
version of the dataset (.xls) in each directory. 


QUESTIONS?
For questions regarding the testbed datasets, contact John Fowler
(john.fowler@asu.edu).  
It is requested that anyone who uploads the testbed datasets send an email
to John Fowler with your name, address, and purpose for using the testbed.
This will allow us to notify you of any changes to the testbed (e.g. when
new datasets are released). It will also be helpful, in general, for us to
know how many people are using the testbed, and for what sorts of
purposes. Also, if you have a data set that you would like to donate,
please contact John Fowler. Thank you. 

Document Revised August 7, 1997.



